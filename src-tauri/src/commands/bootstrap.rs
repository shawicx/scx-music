use crate::db::models::{Playlist, Song};
use crate::db::Db;
use crate::error::AppResult;
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
pub fn get_bootstrap_data(db: tauri::State<'_, Db>) -> AppResult<BootstrapData> {
    let conn = crate::audio::lock_or_recover(&db.0);

    // Songs
    let mut stmt = conn
        .prepare("SELECT id, title, artist, album, duration, duration_secs, quality, file_path, art_gradient, genre, file_size FROM songs ORDER BY created_at")?;
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
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Playlists
    let mut stmt = conn
        .prepare("SELECT id, name, sort_order FROM playlists ORDER BY sort_order")?;
    let playlists = stmt
        .query_map([], |row| {
            Ok(Playlist {
                id: row.get(0)?,
                name: row.get(1)?,
                sort_order: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Playlist-song mappings — 单条查询替代 N+1 prepare（原循环内每个 playlist 重复 prepare 同一 SQL）
    let mut playlist_songs: HashMap<String, Vec<String>> = HashMap::new();
    {
        let mut stmt = conn.prepare(
            "SELECT playlist_id, song_id FROM playlist_songs ORDER BY playlist_id, sort_order",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (playlist_id, song_id) = row?;
            playlist_songs.entry(playlist_id).or_default().push(song_id);
        }
    }

    // Settings
    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    let settings: HashMap<String, String> = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(BootstrapData {
        songs,
        playlists,
        playlist_songs,
        settings,
    })
}
