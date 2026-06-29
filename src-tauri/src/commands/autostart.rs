use crate::error::{AppError, AppResult};
use tauri_plugin_autostart::ManagerExt;

/// 查询开机自启是否已注册到操作系统启动项。
/// 直接读取系统真实状态（macOS LaunchAgent / Windows 注册表 / Linux .desktop），
/// 不信任应用自身的 settings 表，避免与系统状态脱节。
#[tauri::command]
pub fn app_get_autostart(app: tauri::AppHandle) -> AppResult<bool> {
    Ok(app.autolaunch().is_enabled().unwrap_or(false))
}

/// 开启或关闭开机自启。
/// `enabled=true` 写入系统启动项，`enabled=false` 移除。
#[tauri::command]
pub fn app_set_autostart(app: tauri::AppHandle, enabled: bool) -> AppResult<()> {
    let launcher = app.autolaunch();
    if enabled {
        launcher
            .enable()
            .map_err(|e| AppError::OperationFailed(e.to_string()))?;
    } else {
        launcher
            .disable()
            .map_err(|e| AppError::OperationFailed(e.to_string()))?;
    }
    Ok(())
}
