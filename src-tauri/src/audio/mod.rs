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

/// 锁定 Mutex，若中毒（panic 后）取出内部数据继续服务，避免连锁 panic。
/// 音频子系统状态不一致比"整个功能瘫痪"对用户更友好。
///
/// 使用策略：
/// - **进度线程**（本文件 mod.rs）：必须用本函数，线程不能因单次 panic 退出
/// - **后台/守护逻辑**（如 player_set_queue 等批量操作）：用本函数，保持功能可用
/// - **单个 IPC 命令**（player_pause/resume/seek 等单步操作）：保留 `.lock().map_err(...)?`，
///   让前端能感知错误（中毒后用户应看到 toast 而非静默失败）
pub(crate) fn lock_or_recover<T>(
    lock: &std::sync::Mutex<T>,
) -> std::sync::MutexGuard<'_, T> {
    lock.lock().unwrap_or_else(|e| e.into_inner())
}

pub type AudioState = Arc<Mutex<AudioStateInner>>;

pub fn start_progress_thread(state: AudioState, app: AppHandle) -> JoinHandle<()> {
    let stop_flag = lock_or_recover(&state).progress_stop.clone();

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
                        let s = lock_or_recover(&state);
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
                                        let mut s = lock_or_recover(&state);
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

            let (state_kind, progress, duration, song_ended) = {
                let s = lock_or_recover(&state);
                let playing = s.is_sink_playing();
                let payload = s.get_state_payload();
                (
                    s.state.clone(),
                    payload.progress,
                    payload.duration,
                    matches!(s.state, PlaybackState::Playing) && !playing,
                )
            };

            // 仅在 Playing 时推送进度,Paused/Stopped 跳过——
            // 进度条在暂停时已停在前次值,无需重复推送相同数据。
            if matches!(state_kind, PlaybackState::Playing) {
                let _ = app.emit(
                    "audio:progress",
                    serde_json::json!({ "current": progress, "duration": duration }),
                );
            }

            if song_ended {
                let next = {
                    let s = lock_or_recover(&state);
                    s.next_index()
                };

                match next {
                    Some(idx) => {
                        let (play_result, new_payload) = {
                            let mut s = lock_or_recover(&state);
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
                                let _ = app.emit("audio:error", e.to_string());
                            }
                        }
                    }
                    None => {
                        let mut s = lock_or_recover(&state);
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
