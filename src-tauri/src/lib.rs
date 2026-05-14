mod audio;

use std::sync::{Arc, Mutex};

use audio::AudioStateInner;
use lofty::file::{AudioFile as AudioFileTrait, TaggedFileExt};
use lofty::tag::ItemKey;
use serde::Serialize;
use std::fs;
use std::path::Path;
use tauri::Manager;

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
}

#[tauri::command]
fn scan_music_folder(dir_path: String) -> Result<Vec<SongEntry>, String> {
    let path = Path::new(&dir_path);
    if !path.is_dir() {
        return Err("路径不是文件夹".to_string());
    }

    let audio_exts = ["mp3", "flac", "wav", "aac", "ogg", "m4a", "opus", "wma"];
    let mut files = Vec::new();
    scan_dir(path, &audio_exts, &mut files)?;
    Ok(files)
}

fn scan_dir(dir: &Path, exts: &[&str], files: &mut Vec<SongEntry>) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.is_dir() {
            scan_dir(&path, exts, files)?;
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

        let file_path = path.to_string_lossy().to_string();
        let file_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let (title, artist, album, duration_secs) = extract_metadata(&path);
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

        files.push(SongEntry {
            id: format!("{}", files.len()),
            title: title.unwrap_or(file_stem),
            artist: artist.unwrap_or_else(|| "Unknown Artist".to_string()),
            album: album.unwrap_or_else(|| "Unknown Album".to_string()),
            duration,
            duration_secs: secs,
            quality,
            file_path,
        });
    }
    Ok(())
}

fn extract_metadata(path: &Path) -> (Option<String>, Option<String>, Option<String>, Option<f64>) {
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

            (title, artist, album, Some(duration_secs))
        }
        Err(_) => (None, None, None, None),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let audio_state: Arc<Mutex<AudioStateInner>> =
                Arc::new(Mutex::new(AudioStateInner::new()));
            app.manage(audio_state);
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
