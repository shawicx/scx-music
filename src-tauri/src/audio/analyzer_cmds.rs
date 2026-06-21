use tauri::AppHandle;

use super::AudioState;
use crate::error::AppResult;

#[tauri::command]
pub fn analyzer_start(
    app: AppHandle,
    state: tauri::State<'_, AudioState>,
) -> AppResult<()> {
    let s = state.lock().map_err(|e| e.to_string())?;
    s.analyzer.start(app);
    Ok(())
}

#[tauri::command]
pub fn analyzer_stop(state: tauri::State<'_, AudioState>) -> AppResult<()> {
    let s = state.lock().map_err(|e| e.to_string())?;
    s.analyzer.stop();
    Ok(())
}
