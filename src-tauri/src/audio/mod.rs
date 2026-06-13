mod analyzer_cmds;
mod commands;
mod device;
mod engine;
mod types;

pub use analyzer_cmds::*;
pub use commands::*;
pub use device::*;
pub use engine::AudioStateInner;
pub use types::*;

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use tauri::{AppHandle, Emitter};

pub type AudioState = Arc<Mutex<AudioStateInner>>;

pub fn start_progress_thread(state: AudioState, app: AppHandle) -> JoinHandle<()> {
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
    })
}
