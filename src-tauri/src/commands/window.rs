use tauri::Manager;

use crate::error::{AppError, AppResult};

/// 切换主窗口可见性：可见则 hide，不可见则 show + set_focus
/// 仅操作 main 窗口，不影响 mini-player / desktop-lyrics 等其他窗口
#[tauri::command]
pub fn app_toggle_main_window(app: tauri::AppHandle) -> AppResult<()> {
    let win = app
        .get_webview_window("main")
        .ok_or_else(|| AppError::OperationFailed("main window not found".to_string()))?;
    if win.is_visible().unwrap_or(false) {
        win.hide()?;
    } else {
        win.show().and_then(|_| win.set_focus())?;
    }
    Ok(())
}
