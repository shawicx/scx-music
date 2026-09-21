use super::effects::{builtin_presets, find_preset, EqParams, EqPresetDef};
use super::AudioState;
use crate::error::{AppError, AppResult};

#[tauri::command]
pub fn player_list_eq_presets() -> Vec<EqPresetDef> {
    builtin_presets()
}

#[tauri::command]
pub fn player_get_eq(state: tauri::State<'_, AudioState>) -> AppResult<EqParams> {
    let s = state.lock().map_err(|e| e.to_string())?;
    Ok(s.effects.get())
}

/// 切换 EQ 预设。播放中实时生效（块边界 + 系数平滑），未播放时下次播放生效。
/// enabled 状态保留不动（切预设不等于开开关）。
#[tauri::command]
pub fn player_set_eq_preset(
    state: tauri::State<'_, AudioState>,
    preset_id: String,
) -> AppResult<EqParams> {
    let preset = find_preset(&preset_id).ok_or_else(|| {
        AppError::InvalidArgument(format!("Unknown EQ preset: {}", preset_id))
    })?;
    let mut s = state.lock().map_err(|e| e.to_string())?;
    let mut params = s.effects.get();
    params.preset_id = preset.id;
    params.preamp_db = preset.preamp_db;
    params.gains_db = preset.gains_db;
    s.effects.set(params.clone());
    Ok(params)
}

/// EQ 总开关。关闭 = 直通，预设参数保留。
#[tauri::command]
pub fn player_set_eq_enabled(
    state: tauri::State<'_, AudioState>,
    enabled: bool,
) -> AppResult<EqParams> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    let mut params = s.effects.get();
    params.enabled = enabled;
    s.effects.set(params.clone());
    Ok(params)
}
