mod analyzer;
mod audio;
mod commands;
mod db;
mod error;

pub use error::{AppError, AppResult};

use sha2::{Sha256, Digest};
use std::sync::{Arc, Mutex};

use audio::AudioStateInner;
use commands::shortcuts::{self, ShortcutRegistry};
use lofty::file::{AudioFile as AudioFileTrait, TaggedFileExt};
use lofty::tag::ItemKey;
use serde::Serialize;
use std::fs;
use std::path::Path;
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{Builder as ShortcutBuilder, GlobalShortcutExt, ShortcutState};

#[derive(Serialize)]
struct SongEntry {
    id: String,
    title: String,
    artist: String,
    album: String,
    duration: String,
    duration_secs: f64,
    quality: String,
    file_path: String,
    genre: String,
    file_size: u64,
}

/// 递归深度上限：防止循环符号链接或异常深层目录导致栈溢出。
/// 16 层覆盖正常音乐库（罕见深于 16 层）。
const MAX_RECURSION_DEPTH: usize = 16;
/// 扫描文件总数上限：防止超大目录卡死扫描。超出静默截断。
const MAX_FILES_TOTAL: usize = 50_000;

/// 递归扫描音乐文件夹，返回元数据列表（IPC 命令 `scan_music_folder`）。
///
/// 支持的扩展名：mp3 / flac / wav / aac / ogg / m4a / opus / wma。
/// 防护（防止异常输入卡死或栈溢出）：
/// - **深度上限** `MAX_RECURSION_DEPTH = 16`：超过即停止递归（防循环符号链接）
/// - **总数上限** `MAX_FILES_TOTAL = 50_000`：达到即静默截断（防超大目录）
/// - **跳过符号链接**：用 `symlink_metadata`（不跟随）检测并跳过，避免循环链接无限递归
///
/// 非文件夹路径返回 `InvalidArgument` 错误。
#[tauri::command]
fn scan_music_folder(dir_path: String) -> AppResult<Vec<SongEntry>> {
    let path = Path::new(&dir_path);
    if !path.is_dir() {
        return Err(AppError::InvalidArgument("路径不是文件夹".to_string()));
    }

    let audio_exts = ["mp3", "flac", "wav", "aac", "ogg", "m4a", "opus", "wma"];
    let mut files = Vec::new();
    let mut counter = 0usize;
    scan_dir(path, &audio_exts, &mut files, 0, &mut counter)?;
    Ok(files)
}

fn scan_dir(
    dir: &Path,
    exts: &[&str],
    files: &mut Vec<SongEntry>,
    depth: usize,
    counter: &mut usize,
) -> AppResult<()> {
    // 深度上限：超过即停止递归（防止循环符号链接 / 异常深层目录）
    if depth > MAX_RECURSION_DEPTH {
        return Ok(());
    }
    // 总数上限：达到即停止扫描（防止超大目录卡死）
    if *counter >= MAX_FILES_TOTAL {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        // 用 symlink_metadata（不跟随符号链接）判断类型，跳过符号链接，
        // 避免循环符号链接导致的无限递归。
        let meta = fs::symlink_metadata(&path)?;
        if meta.file_type().is_symlink() {
            continue;
        }

        if meta.is_dir() {
            scan_dir(&path, exts, files, depth + 1, counter)?;
            continue;
        }

        if !meta.is_file() {
            continue;
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        if !exts.contains(&ext.as_str()) {
            continue;
        }

        // 总数上限：超出截断并退出
        if *counter >= MAX_FILES_TOTAL {
            eprintln!(
                "[scan] reached MAX_FILES_TOTAL={}, truncating",
                MAX_FILES_TOTAL
            );
            break;
        }
        *counter += 1;

        let file_path = path.to_string_lossy().to_string();
        let file_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let (title, artist, album, duration_secs, genre) = extract_metadata(&path);
        let file_size = meta.len();
        let quality = match ext.as_str() {
            "flac" => "FLAC".to_string(),
            "mp3" => "MP3".to_string(),
            "wav" => "WAV".to_string(),
            "aac" => "AAC".to_string(),
            "ogg" => "OGG".to_string(),
            "m4a" => "M4A (AAC)".to_string(),
            "opus" => "Opus".to_string(),
            _ => ext.to_uppercase(),
        };

        let secs = duration_secs.unwrap_or(0.0);
        let total_secs = secs as u64;
        let duration = format!("{}:{:02}", total_secs / 60, total_secs % 60);

        let id = {
            let mut hasher = Sha256::new();
            hasher.update(file_path.as_bytes());
            let result = hasher.finalize();
            result.iter().map(|b| format!("{:02x}", b)).collect::<String>()
        };

        files.push(SongEntry {
            id,
            title: title.unwrap_or(file_stem),
            artist: artist.unwrap_or_else(|| "Unknown Artist".to_string()),
            album: album.unwrap_or_else(|| "Unknown Album".to_string()),
            duration,
            duration_secs: secs,
            quality,
            file_path,
            genre: genre.unwrap_or_default(),
            file_size,
        });
    }
    Ok(())
}

fn extract_metadata(path: &Path) -> (Option<String>, Option<String>, Option<String>, Option<f64>, Option<String>) {
    match lofty::read_from_path(path) {
        Ok(tagged) => {
            let dur = AudioFileTrait::properties(&tagged).duration();
            let duration_secs = dur.as_secs_f64();

            let tag = tagged.primary_tag();
            let title = tag
                .and_then(|t| t.get_string(&ItemKey::TrackTitle))
                .map(|s| s.to_string());
            let artist = tag
                .and_then(|t| t.get_string(&ItemKey::TrackArtist))
                .map(|s| s.to_string());
            let album = tag
                .and_then(|t| t.get_string(&ItemKey::AlbumTitle))
                .map(|s| s.to_string());
            let genre = tag
                .and_then(|t| t.get_string(&ItemKey::Genre))
                .map(|s| s.to_string());

            (title, artist, album, Some(duration_secs), genre)
        }
        Err(_) => (None, None, None, None, None),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(
            ShortcutBuilder::new().with_handler(|app: &tauri::AppHandle, shortcut, event| {
                if event.state != ShortcutState::Pressed {
                    return;
                }
                let combo = shortcuts::shortcut_to_string(shortcut);
                let registry: tauri::State<ShortcutRegistry> = app.state();
                if let Some(action_id) = registry.get(&combo) {
                    let _ = app.emit("shortcut-triggered", action_id);
                }
            })
            .build(),
        )
        .manage(ShortcutRegistry::default())
        .setup(|app| {
            let mut audio_inner = AudioStateInner::new();

            db::init(&app.handle().clone())?;

            // Load saved output device setting
            let db_state: tauri::State<'_, db::Db> = app.state();
            if let Ok(conn) = db_state.0.lock() {
                if let Ok(name) = conn.query_row(
                    "SELECT value FROM settings WHERE key = 'output_device'",
                    [],
                    |row| row.get::<_, String>(0),
                ) {
                    if !name.is_empty() {
                        audio_inner.set_output_device_name(Some(name));
                    }
                }
            }

            let audio_state: Arc<Mutex<AudioStateInner>> = Arc::new(Mutex::new(audio_inner));
            app.manage(audio_state);

            // 注册启动时应启用的全局快捷键
            if let Err(e) = setup_shortcuts_at_start(app.handle().clone()) {
                eprintln!("[shortcuts] startup registration failed: {e}");
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan_music_folder,
            audio::player_set_queue,
            audio::player_pause,
            audio::player_resume,
            audio::player_stop,
            audio::player_seek,
            audio::player_set_volume,
            audio::player_next,
            audio::player_previous,
            audio::player_set_mode,
            audio::player_get_state,
            audio::player_get_output_devices,
            audio::player_set_output_device,
            audio::player_get_current_device,
            audio::analyzer_start,
            audio::analyzer_stop,
            commands::songs::get_all_songs,
            commands::songs::upsert_songs,
            commands::songs::delete_songs,
            commands::songs::rename_song,
            commands::playlists::get_playlists,
            commands::playlists::create_playlist,
            commands::playlists::rename_playlist,
            commands::playlists::delete_playlist,
            commands::playlists::get_playlist_songs,
            commands::playlists::add_songs_to_playlist,
            commands::playlists::clear_playlist,
            commands::playlists::remove_song_from_playlist,
            commands::settings::get_all_settings,
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::settings::get_system_locale,
            commands::bootstrap::get_bootstrap_data,
            commands::lyrics::get_lyrics,
            commands::lyrics::refresh_lyrics,
            commands::lyrics::set_lyric_offset,
            commands::import_export::export_playlist_m3u,
            commands::import_export::export_playlist_pls,
            commands::import_export::export_backup,
            commands::import_export::import_backup,
            commands::import_export::export_settings,
            commands::import_export::import_settings,
            commands::stats::get_library_stats,
            commands::stats::stats_listening_overview,
            commands::stats::stats_top_songs,
            commands::stats::stats_top_artists,
            commands::stats::stats_genre_distribution,
            commands::stats::stats_trend,
            commands::stats::stats_heatmap,
            commands::stats::stats_hourly_distribution,
            commands::shortcuts::shortcuts_list_defaults,
            commands::shortcuts::shortcuts_register,
            commands::shortcuts::shortcuts_unregister,
            commands::shortcuts::shortcuts_is_registered,
            commands::shortcuts::shortcuts_register_all,
            commands::window::app_toggle_main_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 应用启动时从 settings 表读取快捷键绑定，批量注册 enabled 的项
fn setup_shortcuts_at_start(app: tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let db_state: tauri::State<db::Db> = app.state();
    let conn = crate::audio::lock_or_recover(&db_state.0);

    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    let stored: std::collections::HashMap<String, String> = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?
        .filter_map(|r| r.ok())
        .collect();
    drop(stmt);
    drop(conn); // 释放 Db Mutex

    let defaults = shortcuts::defaults();
    let mut to_register: Vec<(String, String)> = Vec::new();
    for def in &defaults {
        let combo = stored
            .get(&format!("shortcut.{}", def.id))
            .cloned()
            .unwrap_or_else(|| def.combo.clone());
        let enabled = stored
            .get(&format!("shortcut.{}.enabled", def.id))
            .map(|v| v == "true")
            .unwrap_or(def.enabled);
        if enabled && !combo.is_empty() {
            to_register.push((def.id.clone(), combo));
        }
    }

    let registry: tauri::State<ShortcutRegistry> = app.state();
    for (action_id, combo) in to_register {
        let shortcut: tauri_plugin_global_shortcut::Shortcut = match combo.parse() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[shortcuts] skip {action_id}: invalid combo {combo:?}: {e}");
                continue;
            }
        };
        match app.global_shortcut().register(shortcut) {
            Ok(()) => registry.set(combo, action_id),
            Err(e) => eprintln!("[shortcuts] register failed for {action_id}: {e}"),
        }
    }
    Ok(())
}
