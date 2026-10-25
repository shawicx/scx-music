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

/// 原生标题栏占用高度（逻辑像素）。
/// titleBarStyle: Overlay 正常工作时（fullSizeContentView 生效）≈ 0；
/// 个别系统版本会静默忽略 Overlay（如 macOS 26 beta），标题栏重新占位（≈28），
/// 前端据此收起自绘标题栏，避免双标题栏/双拖拽区。
#[tauri::command]
pub fn get_titlebar_inset_height(window: tauri::WebviewWindow) -> AppResult<f64> {
    #[cfg(target_os = "macos")]
    {
        use core_graphics::geometry::CGRect;
        use objc::runtime::Object;
        use objc::{msg_send, sel, sel_impl};

        let ptr = window
            .ns_window()
            .map_err(|e| AppError::OperationFailed(e.to_string()))?;
        unsafe {
            let ns_window = ptr as *mut Object;
            let frame: CGRect = msg_send![ns_window, frame];
            let content: CGRect = msg_send![ns_window, contentLayoutRect];
            Ok((frame.size.height - content.size.height).max(0.0))
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = window; // 参数跨平台保留，非 macOS 不使用
        Ok(0.0)
    }
}
