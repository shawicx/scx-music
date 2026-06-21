use crate::db::Db;
use crate::error::{AppError, AppResult};
use rusqlite::params;
use std::collections::HashMap;

/// set_setting 允许的 key 前缀（动态子键，如 shortcut.media.play-pause）
const ALLOWED_KEY_PREFIXES: &[&str] = &[
    "mini-player.",
    "desktop-lyrics.",
    "shortcut.",
    "lyric.offset.",
];

/// set_setting 允许的精确 key（无子键）
const ALLOWED_EXACT_KEYS: &[&str] = &[
    "language",
    "theme",
    "theme_color",
    "theme_mode",
    "visualization_style",
    "output_device",
    // 库视图状态持久化（useLibrary.ts）
    "activePlaylistId",
    "displayMode",
    "currentSongId",
];

fn validate_setting_key(key: &str) -> AppResult<()> {
    if ALLOWED_EXACT_KEYS.contains(&key) {
        return Ok(());
    }
    for prefix in ALLOWED_KEY_PREFIXES {
        // 前缀匹配要求 key 长度严格大于 prefix，确保存在非空子键
        // （否则 "shortcut." 这样的空后缀会绕过验证）
        if key.len() > prefix.len() && key.starts_with(prefix) {
            return Ok(());
        }
    }
    Err(AppError::InvalidArgument(format!(
        "Unknown setting key (not in whitelist): {}",
        key
    )))
}

#[tauri::command]
pub fn get_system_locale() -> String {
    sys_locale::get_locale().unwrap_or_else(|| "zh-CN".to_string())
}

#[tauri::command]
pub fn get_all_settings(db: tauri::State<'_, Db>) -> AppResult<HashMap<String, String>> {
    let conn = crate::audio::lock_or_recover(&db.0);
    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    let map: HashMap<String, String> = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(map)
}

#[tauri::command]
pub fn get_setting(db: tauri::State<'_, Db>, key: String) -> AppResult<Option<String>> {
    let conn = crate::audio::lock_or_recover(&db.0);
    let result = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get::<_, String>(0),
        )
        .ok();
    Ok(result)
}

#[tauri::command]
pub fn set_setting(db: tauri::State<'_, Db>, key: String, value: String) -> AppResult<()> {
    validate_setting_key(&key)?;
    let conn = crate::audio::lock_or_recover(&db.0);
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        params![key, value],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_keys_are_accepted() {
        assert!(validate_setting_key("language").is_ok());
        assert!(validate_setting_key("theme").is_ok());
        assert!(validate_setting_key("theme_color").is_ok());
        assert!(validate_setting_key("theme_mode").is_ok());
        assert!(validate_setting_key("visualization_style").is_ok());
        assert!(validate_setting_key("output_device").is_ok());
        assert!(validate_setting_key("activePlaylistId").is_ok());
        assert!(validate_setting_key("displayMode").is_ok());
        assert!(validate_setting_key("currentSongId").is_ok());
    }

    #[test]
    fn prefix_keys_are_accepted() {
        assert!(validate_setting_key("mini-player.active").is_ok());
        assert!(validate_setting_key("mini-player.position-x").is_ok());
        assert!(validate_setting_key("desktop-lyrics.locked").is_ok());
        assert!(validate_setting_key("desktop-lyrics.color-current").is_ok());
        assert!(validate_setting_key("shortcut.media.play-pause").is_ok());
        assert!(validate_setting_key("shortcut.media.play-pause.enabled").is_ok());
        assert!(validate_setting_key("lyric.offset.abc123").is_ok());
    }

    #[test]
    fn unknown_keys_are_rejected() {
        assert!(validate_setting_key("foo").is_err());
        assert!(validate_setting_key("evil-key").is_err());
        assert!(validate_setting_key("admin-password").is_err());
        // 前缀必须以点结尾 —— 不带点的"shortcut"应被拒绝（避免绕过）
        assert!(validate_setting_key("shortcut").is_err());
        assert!(validate_setting_key("mini-player").is_err());
        assert!(validate_setting_key("desktop-lyrics").is_err());
        // 空后缀也不应通过
        assert!(validate_setting_key("shortcut.").is_err());
    }

    /// 防止精确 key 与前缀意外重合（如未来有人加了 "shortcut" 精确 key 会破坏"精确先于前缀"语义）
    #[test]
    fn no_exact_key_starts_with_any_prefix() {
        for exact in ALLOWED_EXACT_KEYS {
            for prefix in ALLOWED_KEY_PREFIXES {
                assert!(
                    !exact.starts_with(prefix),
                    "Exact key '{}' starts with prefix '{}' — 会破坏匹配语义",
                    exact,
                    prefix
                );
            }
        }
    }
}
