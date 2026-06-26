use tauri::ipc::Channel;

use super::AudioState;
use crate::error::AppResult;

#[tauri::command]
pub fn analyzer_start(
    state: tauri::State<'_, AudioState>,
    on_data: Channel<Vec<u8>>,
) -> AppResult<()> {
    let s = state.lock().map_err(|e| e.to_string())?;
    s.analyzer.start(on_data);
    Ok(())
}

#[tauri::command]
pub fn analyzer_stop(state: tauri::State<'_, AudioState>) -> AppResult<()> {
    let s = state.lock().map_err(|e| e.to_string())?;
    s.analyzer.stop();
    Ok(())
}
