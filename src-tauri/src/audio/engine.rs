use std::fs::File;
use std::io::BufReader;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

use crate::analyzer::{AnalyzerHandle, TeeSource};

use super::device::{find_device_by_name, try_output_stream_for_device};
use super::types::*;

pub(super) struct AudioEngine {
    pub(super) _stream: OutputStream,
    pub(super) handle: OutputStreamHandle,
    pub(super) sink: Option<Sink>,
}

// SAFETY: OutputStream is safe to send between threads. The lack of Send
// on macOS is due to conservative bounds on CoreAudio callback wrapper types,
// not an actual thread-safety concern. The underlying audio stream is managed
// by the OS and is independent of the thread that created the Rust wrapper.
unsafe impl Send for AudioEngine {}

pub struct AudioStateInner {
    pub(super) engine: Option<AudioEngine>,
    pub(super) queue: Vec<QueuedSong>,
    pub(super) queue_index: usize,
    pub(super) mode: PlaybackMode,
    pub(super) volume: f32,
    pub(super) state: PlaybackState,
    current_song: Option<QueuedSong>,
    pub(super) position_base_secs: f64,
    pub(super) segment_started_at: Option<Instant>,
    pub(super) progress_stop: std::sync::Arc<AtomicBool>,
    pub(super) progress_thread_handle: Option<std::thread::JoinHandle<()>>,
    pub(super) output_device_name: Option<String>,
    pub analyzer: AnalyzerHandle,
}

impl AudioStateInner {
    pub fn new() -> Self {
        Self {
            engine: None,
            queue: Vec::new(),
            queue_index: 0,
            mode: PlaybackMode::Sequential,
            volume: 1.0,
            state: PlaybackState::Stopped,
            current_song: None,
            position_base_secs: 0.0,
            segment_started_at: None,
            progress_stop: std::sync::Arc::new(AtomicBool::new(false)),
            progress_thread_handle: None,
            output_device_name: None,
            analyzer: AnalyzerHandle::new(),
        }
    }

    pub fn set_output_device_name(&mut self, name: Option<String>) {
        self.output_device_name = name;
    }

    pub(super) fn ensure_engine(&mut self) -> Result<(), String> {
        if self.engine.is_some() {
            return Ok(());
        }
        let (stream, handle) = if let Some(ref name) = self.output_device_name {
            match find_device_by_name(name) {
                Ok(device) => match try_output_stream_for_device(&device) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("[audio] Failed to open device '{}': {:?}, falling back to default", name, e);
                        self.output_device_name = None;
                        OutputStream::try_default()
                            .map_err(|e2| format!("No audio output (fallback also failed): device_err={}, default_err={}", e, e2))?
                    }
                },
                Err(e) => {
                    eprintln!("[audio] Device '{}' not found: {}, falling back to default", name, e);
                    self.output_device_name = None;
                    OutputStream::try_default().map_err(|e2| format!("No audio output: {}", e2))?
                }
            }
        } else {
            OutputStream::try_default().map_err(|e| format!("No audio output: {}", e))?
        };
        self.engine = Some(AudioEngine {
            _stream: stream,
            handle,
            sink: None,
        });
        Ok(())
    }

    pub(super) fn rebuild_engine_with_device(&mut self, device_name: Option<String>) -> Result<(), String> {
        let was_paused = matches!(self.state, PlaybackState::Paused);
        let current_index = if self.current_song.is_some() {
            Some(self.queue_index)
        } else {
            None
        };
        let current_position = self.current_position_secs();

        if let Some(engine) = self.engine.take() {
            if let Some(sink) = engine.sink {
                sink.stop();
                sink.detach();
            }
        }

        self.output_device_name = device_name;

        if let Some(index) = current_index {
            self.play_file_at_index(index)?;
            if current_position > 0.0 {
                let _ = self.seek_by_restart(current_position);
            }
            if was_paused {
                self.pause_internal();
            }
        }

        Ok(())
    }

    pub fn current_position_secs(&self) -> f64 {
        self.position_base_secs
            + self
                .segment_started_at
                .map(|t| t.elapsed().as_secs_f64())
                .unwrap_or(0.0)
    }

    pub(super) fn play_file_at_index(&mut self, index: usize) -> Result<(), String> {
        if index >= self.queue.len() {
            self.stop_internal();
            return Ok(());
        }

        let song = self.queue[index].clone();
        self.queue_index = index;

        self.ensure_engine()?;

        {
            let engine = self.engine.as_mut().ok_or("No audio engine")?;
            if let Some(old) = engine.sink.take() {
                old.stop();
                old.detach();
            }
        }

        let sink = {
            let engine = self.engine.as_mut().ok_or("No audio engine")?;
            match Sink::try_new(&engine.handle) {
                Ok(s) => s,
                Err(_) => {
                    let _ = engine;
                    self.engine = None;
                    self.ensure_engine()?;
                    let engine = self.engine.as_mut().ok_or("No audio engine")?;
                    Sink::try_new(&engine.handle).map_err(|e| format!("Sink error: {}", e))?
                }
            }
        };

        let file = File::open(&song.file_path).map_err(|e| format!("File open error: {}", e))?;
        let source =
            Decoder::new(BufReader::new(file)).map_err(|e| format!("Decode error: {}", e))?;

        sink.set_volume(self.volume);
        let teed = TeeSource::new(source.convert_samples::<f32>(), self.analyzer.clone());
        sink.append(teed);
        self.engine.as_mut().ok_or("No audio engine")?.sink = Some(sink);

        self.current_song = Some(song);
        self.state = PlaybackState::Playing;
        self.position_base_secs = 0.0;
        self.segment_started_at = Some(Instant::now());

        Ok(())
    }

    pub fn stop_internal(&mut self) {
        self.progress_stop.store(true, Ordering::Relaxed);
        if let Some(engine) = &mut self.engine {
            if let Some(sink) = engine.sink.take() {
                sink.stop();
                sink.detach();
            }
        }
        self.state = PlaybackState::Stopped;
        self.current_song = None;
        self.position_base_secs = 0.0;
        self.segment_started_at = None;
    }

    pub(super) fn pause_internal(&mut self) {
        if let Some(engine) = &self.engine {
            if let Some(sink) = &engine.sink {
                sink.pause();
            }
        }
        self.position_base_secs = self.current_position_secs();
        self.segment_started_at = None;
        self.state = PlaybackState::Paused;
    }

    pub(super) fn resume_internal(&mut self) {
        if let Some(engine) = &self.engine {
            if let Some(sink) = &engine.sink {
                sink.play();
            }
        }
        self.segment_started_at = Some(Instant::now());
        self.state = PlaybackState::Playing;
    }

    pub(super) fn seek_by_restart(&mut self, position_secs: f64) -> Result<(), String> {
        let song = self.current_song.clone().ok_or("No song playing")?;
        let was_paused = matches!(self.state, PlaybackState::Paused);
        self.ensure_engine()?;
        let engine = self.engine.as_mut().ok_or("No audio engine")?;

        if let Some(old) = engine.sink.take() {
            old.stop();
            old.detach();
        }

        let sink = Sink::try_new(&engine.handle).map_err(|e| format!("Sink error: {}", e))?;
        let file = File::open(&song.file_path).map_err(|e| format!("File open error: {}", e))?;
        let source = Decoder::new(BufReader::new(file))
            .map_err(|e| format!("Decode error: {}", e))?
            .skip_duration(Duration::from_secs_f64(position_secs))
            .convert_samples::<f32>();

        let teed = TeeSource::new(source, self.analyzer.clone());

        sink.set_volume(self.volume);
        sink.append(teed);
        engine.sink = Some(sink);

        if was_paused {
            self.pause_internal();
        } else {
            self.state = PlaybackState::Playing;
        }
        Ok(())
    }

    pub(super) fn next_index(&self) -> Option<usize> {
        if self.queue.is_empty() {
            return None;
        }
        match self.mode {
            PlaybackMode::Sequential => Some((self.queue_index + 1) % self.queue.len()),
            PlaybackMode::RepeatAll => Some((self.queue_index + 1) % self.queue.len()),
            PlaybackMode::RepeatOne => Some(self.queue_index),
            PlaybackMode::Shuffle => {
                if self.queue_index + 1 < self.queue.len() {
                    Some(self.queue_index + 1)
                } else {
                    Some(0)
                }
            }
        }
    }

    pub(super) fn is_sink_playing(&self) -> bool {
        self.engine
            .as_ref()
            .and_then(|e| e.sink.as_ref())
            .map(|s| !s.empty() && !s.is_paused())
            .unwrap_or(false)
    }

    pub fn get_state_payload(&self) -> PlayerStatePayload {
        let duration = self
            .current_song
            .as_ref()
            .map(|s| s.duration_secs)
            .unwrap_or(0.0);
        let progress = if matches!(self.state, PlaybackState::Playing) {
            self.current_position_secs().min(duration)
        } else {
            self.position_base_secs
        };
        PlayerStatePayload {
            current_song: self.current_song.clone(),
            state: self.state.clone(),
            volume: self.volume as f64,
            mode: self.mode.clone(),
            progress,
            duration,
            queue_length: self.queue.len(),
            queue_index: self.queue_index,
        }
    }
}
