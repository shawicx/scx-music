use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

/// 单个动作的默认绑定 — 与前端共享的结构契约
#[derive(Clone, Serialize)]
pub struct ShortcutDefault {
    pub id: String,
    pub combo: String,
    pub enabled: bool,
}

/// 内置动作清单 + 默认绑定
///
/// 注意：combo 字符串格式遵循 Tauri globalShortcut 文档约定：
/// - 修饰符：CommandOrControl / Shift / Alt / Super
/// - 主键：KeyA-KeyZ / Digit0-Digit9 / MediaPlayPause / MediaTrackNext 等
/// - 组合：`Modifier+Modifier+...+Key`，例如 `CommandOrControl+Shift+M`
pub fn defaults() -> Vec<ShortcutDefault> {
    vec![
        ShortcutDefault { id: "media.play-pause".into(),   combo: "MediaPlayPause".into(),           enabled: true  },
        ShortcutDefault { id: "media.next".into(),         combo: "MediaTrackNext".into(),           enabled: true  },
        ShortcutDefault { id: "media.previous".into(),     combo: "MediaTrackPrevious".into(),       enabled: true  },
        ShortcutDefault { id: "media.stop".into(),         combo: "MediaStop".into(),                enabled: false },
        ShortcutDefault { id: "media.volume-up".into(),    combo: "AudioVolumeUp".into(),            enabled: false },
        ShortcutDefault { id: "media.volume-down".into(),  combo: "AudioVolumeDown".into(),          enabled: false },
        ShortcutDefault { id: "media.mute".into(),         combo: "AudioMute".into(),                enabled: false },
        ShortcutDefault { id: "app.mini-player".into(),    combo: "CommandOrControl+Shift+M".into(), enabled: false },
        ShortcutDefault { id: "app.desktop-lyrics".into(), combo: "CommandOrControl+Shift+L".into(), enabled: false },
        ShortcutDefault { id: "app.toggle-window".into(),  combo: "CommandOrControl+Shift+P".into(), enabled: false },
        ShortcutDefault { id: "app.cycle-mode".into(),     combo: "CommandOrControl+Shift+R".into(), enabled: false },
    ]
}

/// 把 Shortcut 序列化为与 DEFAULTS 中相同的字符串形式（用于反查 action_id）
///
/// 注意：`CommandOrControl` 在 global-hotkey 0.8+ 中是平台条件映射：
/// - macOS → SUPER
/// - Windows/Linux → CONTROL
/// 为对称反查，本函数在两种情况下都输出 "CommandOrControl"。
pub fn shortcut_to_string(s: &Shortcut) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mods = s.mods;

    // CommandOrControl 反向识别
    #[cfg(target_os = "macos")]
    let has_cmd_or_ctrl = mods.contains(tauri_plugin_global_shortcut::Modifiers::SUPER);
    #[cfg(not(target_os = "macos"))]
    let has_cmd_or_ctrl = mods.contains(tauri_plugin_global_shortcut::Modifiers::CONTROL);

    if has_cmd_or_ctrl {
        parts.push("CommandOrControl".into());
    }
    if mods.contains(tauri_plugin_global_shortcut::Modifiers::SHIFT) {
        parts.push("Shift".into());
    }
    if mods.contains(tauri_plugin_global_shortcut::Modifiers::ALT) {
        parts.push("Alt".into());
    }
    // SUPER/CONTROL 已通过 CommandOrControl 处理；不再单独输出
    // （Windows 上 SUPER 是 Win 键，可作为独立修饰符，但当前应用未使用）
    #[cfg(not(target_os = "macos"))]
    if mods.contains(tauri_plugin_global_shortcut::Modifiers::SUPER) {
        parts.push("Super".into());
    }
    parts.push(format!("{:?}", s.key));
    parts.join("+")
}

/// Rust 应用状态：保存当前已注册的 combo → action_id 映射
/// 在 setup 时注入到 tauri::Manager，handler 中读取
#[derive(Default)]
pub struct ShortcutRegistry {
    /// combo 字符串 → action_id
    pub combo_to_action: Mutex<HashMap<String, String>>,
}

impl ShortcutRegistry {
    pub fn set(&self, combo: String, action_id: String) {
        self.combo_to_action.lock().unwrap().insert(combo, action_id);
    }
    pub fn remove(&self, combo: &str) {
        self.combo_to_action.lock().unwrap().remove(combo);
    }
    pub fn get(&self, combo: &str) -> Option<String> {
        self.combo_to_action.lock().unwrap().get(combo).cloned()
    }
    pub fn clear(&self) {
        self.combo_to_action.lock().unwrap().clear();
    }
}

#[tauri::command]
pub fn shortcuts_list_defaults() -> Vec<ShortcutDefault> {
    defaults()
}

#[tauri::command]
pub fn shortcuts_register(
    app: tauri::AppHandle,
    registry: tauri::State<'_, ShortcutRegistry>,
    action_id: String,
    combo: String,
) -> Result<(), String> {
    // 如果该 action 已绑定其他 combo，先注销
    // 注意：`Shortcut: FromStr<Err = global_hotkey::HotKeyParseError>`（非插件 Error），
    // 使用 turbofish `parse::<Shortcut>()` 显式绑定类型，便于 map_err 类型推断。
    if let Some(old_combo) = find_combo_by_action(&registry, &action_id) {
        let shortcut = old_combo.parse::<Shortcut>().map_err(|e| e.to_string())?;
        app.global_shortcut().unregister(shortcut).map_err(|e| e.to_string())?;
        registry.remove(&old_combo);
    }

    let shortcut = combo.parse::<Shortcut>().map_err(|e| e.to_string())?;
    app.global_shortcut().register(shortcut).map_err(|e| e.to_string())?;
    registry.set(combo, action_id);
    Ok(())
}

#[tauri::command]
pub fn shortcuts_unregister(
    app: tauri::AppHandle,
    registry: tauri::State<'_, ShortcutRegistry>,
    action_id: String,
) -> Result<(), String> {
    if let Some(combo) = find_combo_by_action(&registry, &action_id) {
        let shortcut = combo.parse::<Shortcut>().map_err(|e| e.to_string())?;
        let _ = app.global_shortcut().unregister(shortcut);
        registry.remove(&combo);
    }
    Ok(())
}

#[tauri::command]
pub fn shortcuts_is_registered(
    app: tauri::AppHandle,
    combo: String,
) -> Result<bool, String> {
    let shortcut = combo.parse::<Shortcut>().map_err(|e| e.to_string())?;
    Ok(app.global_shortcut().is_registered(shortcut))
}

#[tauri::command]
pub fn shortcuts_register_all(
    app: tauri::AppHandle,
    registry: tauri::State<'_, ShortcutRegistry>,
    bindings: Vec<(String, String)>,
) -> Result<(), String> {
    for (action_id, combo) in bindings {
        if combo.is_empty() { continue; }
        let shortcut = match combo.parse::<Shortcut>() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[shortcuts] skip {action_id}: invalid combo {combo:?}: {e}");
                continue;
            }
        };
        match app.global_shortcut().register(shortcut) {
            Ok(()) => registry.set(combo, action_id),
            Err(e) => eprintln!("[shortcuts] register failed for {action_id} ({combo}): {e}"),
        }
    }
    Ok(())
}

/// 在 registry 中反查 action_id 对应的当前 combo
fn find_combo_by_action(registry: &tauri::State<'_, ShortcutRegistry>, action_id: &str) -> Option<String> {
    let map = registry.combo_to_action.lock().unwrap();
    for (combo, id) in map.iter() {
        if id == action_id {
            return Some(combo.clone());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_ids_are_unique() {
        let defs = defaults();
        let mut ids: Vec<&str> = defs.iter().map(|d| d.id.as_str()).collect();
        ids.sort();
        let initial_len = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), initial_len, "duplicate action_id in defaults");
    }

    #[test]
    fn defaults_count_is_eleven() {
        // 3 个默认开 + 8 个默认关 = 11
        assert_eq!(defaults().len(), 11);
    }

    #[test]
    fn defaults_enabled_flags_match_design() {
        let defs: std::collections::HashMap<String, ShortcutDefault> =
            defaults().into_iter().map(|d| (d.id.clone(), d)).collect();
        assert!(defs["media.play-pause"].enabled);
        assert!(defs["media.next"].enabled);
        assert!(defs["media.previous"].enabled);
        // 其余全部应为 false
        for id in [
            "media.stop", "media.volume-up", "media.volume-down", "media.mute",
            "app.mini-player", "app.desktop-lyrics", "app.toggle-window", "app.cycle-mode",
        ] {
            assert!(!defs[id].enabled, "{id} should be disabled by default");
        }
    }

    #[test]
    fn defaults_combos_match_design() {
        let defs: std::collections::HashMap<String, String> =
            defaults().into_iter().map(|d| (d.id, d.combo)).collect();
        assert_eq!(defs["media.play-pause"], "MediaPlayPause");
        assert_eq!(defs["media.next"], "MediaTrackNext");
        assert_eq!(defs["media.previous"], "MediaTrackPrevious");
        assert_eq!(defs["app.mini-player"], "CommandOrControl+Shift+M");
        assert_eq!(defs["app.desktop-lyrics"], "CommandOrControl+Shift+L");
        assert_eq!(defs["app.toggle-window"], "CommandOrControl+Shift+P");
        assert_eq!(defs["app.cycle-mode"], "CommandOrControl+Shift+R");
    }

    #[test]
    fn defaults_combos_are_unique() {
        let defs = defaults();
        let mut combos: Vec<&str> = defs.iter().map(|d| d.combo.as_str()).collect();
        combos.sort();
        let initial_len = combos.len();
        combos.dedup();
        assert_eq!(combos.len(), initial_len, "duplicate combo in defaults");
    }
}
