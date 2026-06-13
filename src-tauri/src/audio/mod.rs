mod analyzer_cmds;
mod commands;
mod device;
mod engine;
mod tracker;
mod types;

pub use analyzer_cmds::*;
pub use commands::*;
pub use device::*;
pub use engine::AudioStateInner;
pub use tracker::*;
pub use types::*;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};

pub type AudioState = Arc<Mutex<AudioStateInner>>;

pub fn start_progress_thread(state: AudioState, app: AppHandle) -> JoinHandle<()> {
    let stop_flag = state.lock().unwrap().progress_stop.clone();

    thread::spawn(move || {
        static FLUSH_COUNTER: AtomicU64 = AtomicU64::new(0);

        loop {
            thread::sleep(Duration::from_millis(500));
            if stop_flag.load(Ordering::Relaxed) {
                break;
            }

            // Auto-flush play session every ~30 seconds (60 iterations at 500ms)
            // Only writes duration, does NOT touch play_count
            {
                let count = FLUSH_COUNTER.fetch_add(1, Ordering::Relaxed);
                if count % 60 == 59 {
                    let session_data = {
                        let s = state.lock().unwrap();
                        s.play_session
                            .as_ref()
                            .map(|sess| (sess.song_id.clone(), sess.song_duration_secs, sess.total_secs(), sess.total_listened_secs))
                    };
                    if let Some((song_id, song_dur, total_secs, total_listened)) = session_data {
                        if total_secs >= 5.0 {
                            if let Some(db) = app.try_state::<crate::db::Db>() {
                                if let Ok(conn) = db.0.lock() {
                                    let temp = PlaySession {
                                        song_id,
                                        song_duration_secs: song_dur,
                                        started_at: None,
                                        accumulated_secs: total_secs,
                                        total_listened_secs: 0.0,
                                    };
                                    if flush_session(&conn, &temp).unwrap_or(false) {
                                        let mut s = state.lock().unwrap();
                                        if let Some(ref mut sess) = s.play_session {
                                            sess.total_listened_secs = total_listened + total_secs;
                                            sess.accumulated_secs = 0.0;
                                            sess.started_at = Some(std::time::Instant::now());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
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
                            let result = s.play_file_at_index(idx, Some(&app));
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
                        s.stop_internal(Some(&app));
                        let payload = s.get_state_payload();
                        drop(s);
                        let _ = app.emit("audio:state_change", &payload);
                    }
                }
            }
        }
    })
}
