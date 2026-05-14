use std::fs::File;
use std::io::BufReader;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use rand::Rng;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackMode {
    Sequential,
    RepeatAll,
    RepeatOne,
    Shuffle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueuedSong {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_secs: f64,
    pub quality: String,
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerStatePayload {
    pub current_song: Option<QueuedSong>,
    pub state: PlaybackState,
    pub volume: f64,
    pub mode: PlaybackMode,
    pub progress: f64,
    pub duration: f64,
    pub queue_length: usize,
    pub queue_index: usize,
}

struct AudioEngine {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    sink: Option<Sink>,
}

// SAFETY: OutputStream is safe to send between threads. The lack of Send
// on macOS is due to conservative bounds on CoreAudio callback wrapper types,
// not an actual thread-safety concern. The underlying audio stream is managed
// by the OS and is independent of the thread that created the Rust wrapper.
unsafe impl Send for AudioEngine {}

pub struct AudioStateInner {
    engine: Option<AudioEngine>,
    queue: Vec<QueuedSong>,
    queue_index: usize,
    mode: PlaybackMode,
    volume: f32,
    state: PlaybackState,
    current_song: Option<QueuedSong>,
    position_base_secs: f64,
    segment_started_at: Option<Instant>,
    progress_stop: Arc<AtomicBool>,
}

pub type AudioState = Arc<Mutex<AudioStateInner>>;

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
            progress_stop: Arc::new(AtomicBool::new(false)),
        }
    }

    fn ensure_engine(&mut self) -> Result<(), String> {
        if self.engine.is_some() {
            return Ok(());
        }
        let (stream, handle) =
            OutputStream::try_default().map_err(|e| format!("No audio output: {}", e))?;
        self.engine = Some(AudioEngine {
            _stream: stream,
            handle,
            sink: None,
        });
        Ok(())
    }

    pub fn current_position_secs(&self) -> f64 {
        self.position_base_secs
            + self
                .segment_started_at
                .map(|t| t.elapsed().as_secs_f64())
                .unwrap_or(0.0)
    }

    fn play_file_at_index(&mut self, index: usize) -> Result<(), String> {
        if index >= self.queue.len() {
            self.stop_internal();
            return Ok(());
        }

        let song = self.queue[index].clone();
        self.queue_index = index;

        self.ensure_engine()?;
        let engine = self.engine.as_mut().ok_or("No audio engine")?;

        if let Some(old) = engine.sink.take() {
            old.stop();
            old.detach();
        }

        let sink = Sink::try_new(&engine.handle).map_err(|e| format!("Sink error: {}", e))?;

        let file = File::open(&song.file_path).map_err(|e| format!("File open error: {}", e))?;
        let source =
            Decoder::new(BufReader::new(file)).map_err(|e| format!("Decode error: {}", e))?;

        sink.set_volume(self.volume);
        sink.append(source);
        engine.sink = Some(sink);

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

    fn pause_internal(&mut self) {
        if let Some(engine) = &self.engine {
            if let Some(sink) = &engine.sink {
                sink.pause();
            }
        }
        self.position_base_secs = self.current_position_secs();
        self.segment_started_at = None;
        self.state = PlaybackState::Paused;
    }

    fn resume_internal(&mut self) {
        if let Some(engine) = &self.engine {
            if let Some(sink) = &engine.sink {
                sink.play();
            }
        }
        self.segment_started_at = Some(Instant::now());
        self.state = PlaybackState::Playing;
    }

    fn seek_by_restart(&mut self, position_secs: f64) -> Result<(), String> {
        let song = self.current_song.clone().ok_or("No song playing")?;
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
            .skip_duration(Duration::from_secs_f64(position_secs));

        sink.set_volume(self.volume);
        sink.append(source);
        engine.sink = Some(sink);
        self.state = PlaybackState::Playing;
        Ok(())
    }

    fn next_index(&self) -> Option<usize> {
        if self.queue.is_empty() {
            return None;
        }
        match self.mode {
            PlaybackMode::Sequential => {
                if self.queue_index + 1 < self.queue.len() {
                    Some(self.queue_index + 1)
                } else {
                    None
                }
            }
            PlaybackMode::RepeatAll => Some((self.queue_index + 1) % self.queue.len()),
            PlaybackMode::RepeatOne => Some(self.queue_index),
            PlaybackMode::Shuffle => {
                if self.queue.len() == 1 {
                    Some(0)
                } else {
                    let mut rng = rand::thread_rng();
                    loop {
                        let i = rng.gen_range(0..self.queue.len());
                        if i != self.queue_index {
                            return Some(i);
                        }
                    }
                }
            }
        }
    }

    fn is_sink_playing(&self) -> bool {
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

pub fn start_progress_thread(state: AudioState, app: AppHandle) {
    let stop_flag = state.lock().unwrap().progress_stop.clone();

    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(500));
            if stop_flag.load(Ordering::Relaxed) {
                break;
            }

            let (progress, duration, song_ended) = {
                let s = state.lock().unwrap();
                let playing = s.is_sink_playing();
                let payload = s.get_state_payload();
                (
                    payload.progress,
                    payload.duration,
                    matches!(s.state, PlaybackState::Playing) && !playing,
                )
            };

            let _ = app.emit(
                "audio:progress",
                serde_json::json!({ "current": progress, "duration": duration }),
            );

            if song_ended {
                let next = {
                    let s = state.lock().unwrap();
                    s.next_index()
                };

                match next {
                    Some(idx) => {
                        let (play_result, new_payload) = {
                            let mut s = state.lock().unwrap();
                            let result = s.play_file_at_index(idx);
                            let payload = s.get_state_payload();
                            (result, payload)
                        };
                        match play_result {
                            Ok(()) => {
                                let _ = app.emit("audio:track_change", &new_payload.current_song);
                                let _ = app.emit("audio:state_change", &new_payload);
                            }
                            Err(e) => {
                                let _ = app.emit("audio:error", e);
                            }
                        }
                    }
                    None => {
                        let mut s = state.lock().unwrap();
                        s.stop_internal();
                        let payload = s.get_state_payload();
                        drop(s);
                        let _ = app.emit("audio:state_change", &payload);
                    }
                }
            }
        }
    });
}

// ---- Tauri Commands ----

#[tauri::command]
pub fn player_set_queue(
    app: AppHandle,
    state: tauri::State<'_, AudioState>,
    songs: Vec<QueuedSong>,
    index: usize,
) -> Result<(), String> {
    let arc: AudioState = (*state).clone();
    {
        let mut s = arc.lock().unwrap();
        s.progress_stop.store(false, Ordering::Relaxed);
        s.queue = songs;
        s.play_file_at_index(index)?;
        let payload = s.get_state_payload();
        drop(s);
        let _ = app.emit("audio:track_change", &payload.current_song);
        let _ = app.emit("audio:state_change", &payload);
    }
    start_progress_thread(arc, app);
    Ok(())
}

#[tauri::command]
pub fn player_pause(
    app: AppHandle,
    state: tauri::State<'_, AudioState>,
) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    s.pause_internal();
    let payload = s.get_state_payload();
    drop(s);
    let _ = app.emit("audio:state_change", &payload);
    Ok(())
}

#[tauri::command]
pub fn player_resume(
    app: AppHandle,
    state: tauri::State<'_, AudioState>,
) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    s.resume_internal();
    let payload = s.get_state_payload();
    drop(s);
    let _ = app.emit("audio:state_change", &payload);
    Ok(())
}

#[tauri::command]
pub fn player_stop(state: tauri::State<'_, AudioState>) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    s.stop_internal();
    Ok(())
}

#[tauri::command]
pub fn player_seek(
    state: tauri::State<'_, AudioState>,
    position_secs: f64,
) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;

    let seeked = if let Some(engine) = &s.engine {
        if let Some(sink) = &engine.sink {
            sink.try_seek(Duration::from_secs_f64(position_secs)).is_ok()
        } else {
            false
        }
    } else {
        false
    };

    if !seeked {
        s.seek_by_restart(position_secs)?;
    }

    s.position_base_secs = position_secs;
    s.segment_started_at = Some(Instant::now());
    Ok(())
}

#[tauri::command]
pub fn player_set_volume(
    state: tauri::State<'_, AudioState>,
    volume: f64,
) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    s.volume = volume as f32;
    if let Some(engine) = &s.engine {
        if let Some(sink) = &engine.sink {
            sink.set_volume(volume as f32);
        }
    }
    Ok(())
}

#[tauri::command]
pub fn player_next(app: AppHandle, state: tauri::State<'_, AudioState>) -> Result<(), String> {
    let arc: AudioState = (*state).clone();
    let next = {
        let s = arc.lock().unwrap();
        s.next_index()
    };
    if let Some(idx) = next {
        let payload;
        {
            let mut s = arc.lock().unwrap();
            s.play_file_at_index(idx)?;
            payload = s.get_state_payload();
        }
        let _ = app.emit("audio:track_change", &payload.current_song);
        let _ = app.emit("audio:state_change", &payload);
    }
    Ok(())
}

#[tauri::command]
pub fn player_previous(app: AppHandle, state: tauri::State<'_, AudioState>) -> Result<(), String> {
    let arc: AudioState = (*state).clone();
    let prev = {
        let s = arc.lock().unwrap();
        if s.queue_index > 0 {
            Some(s.queue_index - 1)
        } else if matches!(s.mode, PlaybackMode::RepeatAll) && !s.queue.is_empty() {
            Some(s.queue.len() - 1)
        } else {
            None
        }
    };
    if let Some(idx) = prev {
        let payload;
        {
            let mut s = arc.lock().unwrap();
            s.play_file_at_index(idx)?;
            payload = s.get_state_payload();
        }
        let _ = app.emit("audio:track_change", &payload.current_song);
        let _ = app.emit("audio:state_change", &payload);
    }
    Ok(())
}

#[tauri::command]
pub fn player_set_mode(
    state: tauri::State<'_, AudioState>,
    mode: PlaybackMode,
) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    s.mode = mode;
    Ok(())
}

#[tauri::command]
pub fn player_get_state(state: tauri::State<'_, AudioState>) -> Result<PlayerStatePayload, String> {
    let s = state.lock().map_err(|e| e.to_string())?;
    Ok(s.get_state_payload())
}
