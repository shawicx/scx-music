use crate::db::models::{Playlist, Song};
use crate::db::Db;
use rusqlite::params;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapData {
    pub songs: Vec<Song>,
    pub playlists: Vec<Playlist>,
    pub playlist_songs: HashMap<String, Vec<String>>,
    pub settings: HashMap<String, String>,
}

#[tauri::command]
pub fn get_bootstrap_data(db: tauri::State<'_, Db>) -> Result<BootstrapData, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Songs
    let mut stmt = conn
        .prepare("SELECT id, title, artist, album, duration, duration_secs, quality, file_path, art_gradient, genre, file_size FROM songs ORDER BY created_at")
        .map_err(|e| e.to_string())?;
    let songs = stmt
        .query_map([], |row| {
            Ok(Song {
                id: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                album: row.get(3)?,
                duration: row.get(4)?,
                duration_secs: row.get(5)?,
                quality: row.get(6)?,
                file_path: row.get(7)?,
                art_gradient: row.get(8)?,
                genre: row.get(9)?,
                file_size: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // Playlists
    let mut stmt = conn
        .prepare("SELECT id, name, sort_order FROM playlists ORDER BY sort_order")
        .map_err(|e| e.to_string())?;
    let playlists = stmt
        .query_map([], |row| {
            Ok(Playlist {
                id: row.get(0)?,
                name: row.get(1)?,
                sort_order: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // Playlist-song mappings (all in one DB lock)
    let mut playlist_songs = HashMap::new();
    for p in &playlists {
        let mut stmt = conn
            .prepare(
                "SELECT song_id FROM playlist_songs WHERE playlist_id = ?1 ORDER BY sort_order",
            )
            .map_err(|e| e.to_string())?;
        let ids: Vec<String> = stmt
            .query_map(params![p.id], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        playlist_songs.insert(p.id.clone(), ids);
    }

    // Settings
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings")
        .map_err(|e| e.to_string())?;
    let settings: HashMap<String, String> = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(BootstrapData {
        songs,
        playlists,
        playlist_songs,
        settings,
    })
}
