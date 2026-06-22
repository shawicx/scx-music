use std::fs::File;
use std::io::BufReader;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use rand::seq::SliceRandom;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use tauri::Manager;

use crate::analyzer::{AnalyzerHandle, TeeSource};

use super::device::{find_device_by_name, try_output_stream_for_device};
use super::tracker::{finalize_session, PlaySession};
use super::types::*;
use crate::error::{AppError, AppResult};

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
    pub play_session: Option<PlaySession>,
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
            play_session: None,
        }
    }

    pub fn set_output_device_name(&mut self, name: Option<String>) {
        self.output_device_name = name;
    }

    pub(super) fn ensure_engine(&mut self) -> AppResult<()> {
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

    pub(super) fn rebuild_engine_with_device(&mut self, device_name: Option<String>) -> AppResult<()> {
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
            self.play_file_at_index(index, None)?;
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

    pub(super) fn play_file_at_index(
        &mut self,
        index: usize,
        app: Option<&tauri::AppHandle>,
    ) -> AppResult<()> {
        if index >= self.queue.len() {
            self.stop_internal(app);
            return Ok(());
        }

        // Finalize old play session
        if let Some(session) = self.play_session.take() {
            if let Some(app) = app {
                if let Some(db) = app.try_state::<crate::db::Db>() {
                    if let Ok(conn) = db.0.lock() {
                        let _ = finalize_session(&conn, &session);
                    }
                }
            }
        }

        let song = self.queue[index].clone();
        self.play_session = Some(PlaySession::new(song.id.clone(), song.duration_secs));
        self.queue_index = index;

        self.ensure_engine()?;

        {
            let engine = self.engine.as_mut().ok_or(AppError::AudioPlayback("No audio engine".to_string()))?;
            if let Some(old) = engine.sink.take() {
                old.stop();
                old.detach();
            }
        }

        let sink = {
            let engine = self.engine.as_mut().ok_or(AppError::AudioPlayback("No audio engine".to_string()))?;
            match Sink::try_new(&engine.handle) {
                Ok(s) => s,
                Err(_) => {
                    let _ = engine;
                    self.engine = None;
                    self.ensure_engine()?;
                    let engine = self.engine.as_mut().ok_or(AppError::AudioPlayback("No audio engine".to_string()))?;
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
        self.engine.as_mut().ok_or(AppError::AudioPlayback("No audio engine".to_string()))?.sink = Some(sink);

        self.current_song = Some(song);
        self.state = PlaybackState::Playing;
        self.position_base_secs = 0.0;
        self.segment_started_at = Some(Instant::now());

        Ok(())
    }

    pub fn stop_internal(&mut self, app: Option<&tauri::AppHandle>) {
        // Finalize play session
        if let Some(session) = self.play_session.take() {
            if let Some(app) = app {
                if let Some(db) = app.try_state::<crate::db::Db>() {
                    if let Ok(conn) = db.0.lock() {
                        let _ = finalize_session(&conn, &session);
                    }
                }
            }
        }

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
        if let Some(ref mut s) = self.play_session {
            s.pause();
        }
    }

    pub(super) fn resume_internal(&mut self) {
        if let Some(engine) = &self.engine {
            if let Some(sink) = &engine.sink {
                sink.play();
            }
        }
        self.segment_started_at = Some(Instant::now());
        self.state = PlaybackState::Playing;
        if let Some(ref mut s) = self.play_session {
            s.resume();
        }
    }

    pub(super) fn seek_by_restart(&mut self, position_secs: f64) -> AppResult<()> {
        let song = self.current_song.clone().ok_or(AppError::AudioPlayback("No song playing".to_string()))?;
        let was_paused = matches!(self.state, PlaybackState::Paused);
        self.ensure_engine()?;
        let engine = self.engine.as_mut().ok_or(AppError::AudioPlayback("No audio engine".to_string()))?;

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
            // 顺序播放：到末尾停止（返回 None）。当前歌曲自然播完后，
            // 进度线程检测到 None 不切歌，歌曲停止；这是 Sequential 的期望语义。
            PlaybackMode::Sequential => {
                if self.queue_index + 1 < self.queue.len() {
                    Some(self.queue_index + 1)
                } else {
                    None
                }
            }
            // 列表循环：回到第一首
            PlaybackMode::RepeatAll => Some((self.queue_index + 1) % self.queue.len()),
            // 单曲循环：保持当前
            PlaybackMode::RepeatOne => Some(self.queue_index),
            // 随机：避免连续两次播放同一首（len==1 时退化为单曲循环）
            PlaybackMode::Shuffle => {
                if self.queue.len() == 1 {
                    return Some(0);
                }
                let candidates: Vec<usize> = (0..self.queue.len())
                    .filter(|&i| i != self.queue_index)
                    .collect();
                candidates.choose(&mut rand::thread_rng()).copied()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::types::{PlaybackMode, QueuedSong};

    fn make_queue(len: usize) -> Vec<QueuedSong> {
        (0..len)
            .map(|i| QueuedSong {
                id: format!("song-{i}"),
                title: format!("T{i}"),
                artist: "A".into(),
                album: "Al".into(),
                duration_secs: 0.0,
                quality: "lossy".into(),
                file_path: format!("/p/{i}"),
            })
            .collect()
    }

    #[test]
    fn sequential_returns_next_until_end() {
        let mut state = AudioStateInner::new();
        state.queue = make_queue(3);
        state.mode = PlaybackMode::Sequential;

        state.queue_index = 0;
        assert_eq!(state.next_index(), Some(1));

        state.queue_index = 1;
        assert_eq!(state.next_index(), Some(2));

        // 末尾返回 None（停止）
        state.queue_index = 2;
        assert_eq!(state.next_index(), None);
    }

    #[test]
    fn sequential_empty_queue_returns_none() {
        let mut state = AudioStateInner::new();
        state.mode = PlaybackMode::Sequential;
        assert_eq!(state.next_index(), None);
    }

    #[test]
    fn repeat_all_wraps_around() {
        let mut state = AudioStateInner::new();
        state.queue = make_queue(3);
        state.mode = PlaybackMode::RepeatAll;

        state.queue_index = 1;
        assert_eq!(state.next_index(), Some(2));

        // 末尾回到 0
        state.queue_index = 2;
        assert_eq!(state.next_index(), Some(0));
    }

    #[test]
    fn repeat_one_stays_at_current() {
        let mut state = AudioStateInner::new();
        state.queue = make_queue(3);
        state.mode = PlaybackMode::RepeatOne;

        state.queue_index = 1;
        assert_eq!(state.next_index(), Some(1));
    }

    #[test]
    fn shuffle_never_returns_current_and_single_track_returns_zero() {
        let mut state = AudioStateInner::new();
        state.queue = make_queue(5);
        state.mode = PlaybackMode::Shuffle;

        // 多元素：任一起点都不返回自身，且落在合法范围
        for start in 0..5 {
            state.queue_index = start;
            let next = state.next_index().expect("shuffle should return Some");
            assert_ne!(next, start, "shuffle 不应返回当前索引");
            assert!(next < 5, "shuffle 索引越界");
        }

        // 单元素：退化为返回 0
        state.queue = make_queue(1);
        state.queue_index = 0;
        assert_eq!(state.next_index(), Some(0));
    }
}
