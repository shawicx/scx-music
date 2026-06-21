use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter};

use super::lock_or_recover;
use super::types::*;
use super::AudioState;
use crate::error::AppResult;

#[tauri::command]
pub fn player_set_queue(
    app: AppHandle,
    state: tauri::State<'_, AudioState>,
    songs: Vec<QueuedSong>,
    index: usize,
) -> AppResult<()> {
    let arc: AudioState = (*state).clone();
    {
        let mut s = lock_or_recover(&arc);
        s.progress_stop.store(true, Ordering::Relaxed);
        let old_handle = s.progress_thread_handle.take();
        drop(s);
        if let Some(handle) = old_handle {
            let _ = handle.join();
        }
        let mut s = lock_or_recover(&arc);
        s.progress_stop.store(false, Ordering::Relaxed);
        s.queue = songs;
        s.play_file_at_index(index, Some(&app))?;
        let payload = s.get_state_payload();
        drop(s);
        let _ = app.emit("audio:track_change", &payload.current_song);
        let _ = app.emit("audio:state_change", &payload);
    }
    let handle = super::start_progress_thread(arc.clone(), app);
    lock_or_recover(&arc).progress_thread_handle = Some(handle);
    Ok(())
}

#[tauri::command]
pub fn player_pause(
    app: AppHandle,
    state: tauri::State<'_, AudioState>,
) -> AppResult<()> {
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
) -> AppResult<()> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    s.resume_internal();
    let payload = s.get_state_payload();
    drop(s);
    let _ = app.emit("audio:state_change", &payload);
    Ok(())
}

#[tauri::command]
pub fn player_stop(app: AppHandle, state: tauri::State<'_, AudioState>) -> AppResult<()> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    s.stop_internal(Some(&app));
    Ok(())
}

#[tauri::command]
pub fn player_seek(
    state: tauri::State<'_, AudioState>,
    position_secs: f64,
) -> AppResult<()> {
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
) -> AppResult<()> {
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
pub fn player_next(app: AppHandle, state: tauri::State<'_, AudioState>) -> AppResult<()> {
    let arc: AudioState = (*state).clone();
    let next = {
        let s = lock_or_recover(&arc);
        s.next_index()
    };
    if let Some(idx) = next {
        let payload;
        {
            let mut s = lock_or_recover(&arc);
            s.play_file_at_index(idx, Some(&app))?;
            payload = s.get_state_payload();
        }
        let _ = app.emit("audio:track_change", &payload.current_song);
        let _ = app.emit("audio:state_change", &payload);
    }
    Ok(())
}

#[tauri::command]
pub fn player_previous(app: AppHandle, state: tauri::State<'_, AudioState>) -> AppResult<()> {
    let arc: AudioState = (*state).clone();
    let prev = {
        let s = lock_or_recover(&arc);
        if s.queue_index > 0 {
            Some(s.queue_index - 1)
        } else if !s.queue.is_empty() {
            Some(s.queue.len() - 1)
        } else {
            None
        }
    };
    if let Some(idx) = prev {
        let payload;
        {
            let mut s = lock_or_recover(&arc);
            s.play_file_at_index(idx, Some(&app))?;
            payload = s.get_state_payload();
        }
        let _ = app.emit("audio:track_change", &payload.current_song);
        let _ = app.emit("audio:state_change", &payload);
    }
    Ok(())
}

#[tauri::command]
pub fn player_set_mode(
    app: AppHandle,
    state: tauri::State<'_, AudioState>,
    mode: PlaybackMode,
) -> AppResult<()> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    s.mode = mode;
    let payload = s.get_state_payload();
    drop(s);
    let _ = app.emit("audio:state_change", &payload);
    Ok(())
}

#[tauri::command]
pub fn player_get_state(state: tauri::State<'_, AudioState>) -> AppResult<PlayerStatePayload> {
    let s = state.lock().map_err(|e| e.to_string())?;
    Ok(s.get_state_payload())
}
