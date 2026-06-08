use crate::db::models::{Lyric, Playlist, PlaylistSong, Song};
use crate::db::Db;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

// --- Backup data structures ---

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackupData {
    songs: Vec<Song>,
    playlists: Vec<Playlist>,
    playlist_songs: Vec<PlaylistSong>,
    settings: HashMap<String, String>,
    lyrics: Vec<Lyric>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BackupFile {
    version: u32,
    app: String,
    exported_at: u64,
    data: BackupData,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackupFileInput {
    version: u32,
    data: BackupData,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub songs_imported: usize,
    pub playlists_imported: usize,
    pub lyrics_imported: usize,
}

// --- Playlist export helpers ---

fn get_playlist_songs_from_db(
    conn: &rusqlite::Connection,
    playlist_id: &str,
) -> Result<Vec<Song>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.title, s.artist, s.album, s.duration, s.duration_secs, s.quality, s.file_path, s.art_gradient
             FROM songs s
             INNER JOIN playlist_songs ps ON s.id = ps.song_id
             WHERE ps.playlist_id = ?1
             ORDER BY ps.sort_order",
        )
        .map_err(|e| e.to_string())?;
    let result: Vec<Song> = stmt
        .query_map(params![playlist_id], |row| {
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
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(result)
}

// --- Commands ---

#[tauri::command]
pub fn export_playlist_m3u(
    db: tauri::State<'_, Db>,
    playlist_id: String,
    save_path: String,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let songs = get_playlist_songs_from_db(&conn, &playlist_id)?;

    let mut content = String::from("#EXTM3U\n");
    for s in &songs {
        let secs = s.duration_secs as i64;
        content.push_str(&format!(
            "#EXTINF:{},{} - {}\n{}\n",
            secs, s.artist, s.title, s.file_path
        ));
    }

    fs::write(&save_path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_playlist_pls(
    db: tauri::State<'_, Db>,
    playlist_id: String,
    save_path: String,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let songs = get_playlist_songs_from_db(&conn, &playlist_id)?;

    let mut content = String::from("[playlist]\n");
    for (i, s) in songs.iter().enumerate() {
        let n = i + 1;
        let secs = s.duration_secs as i64;
        content.push_str(&format!(
            "File{}={}\nTitle{}={} - {}\nLength{}={}\n",
            n, s.file_path, n, s.artist, s.title, n, secs
        ));
    }
    content.push_str(&format!(
        "NumberOfEntries={}\nVersion=2\n",
        songs.len()
    ));

    fs::write(&save_path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_backup(
    db: tauri::State<'_, Db>,
    save_path: String,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Songs
    let mut stmt = conn
        .prepare("SELECT id, title, artist, album, duration, duration_secs, quality, file_path, art_gradient FROM songs ORDER BY created_at")
        .map_err(|e| e.to_string())?;
    let songs: Vec<Song> = stmt
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
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // Playlists
    let mut stmt = conn
        .prepare("SELECT id, name, sort_order FROM playlists ORDER BY sort_order")
        .map_err(|e| e.to_string())?;
    let playlists: Vec<Playlist> = stmt
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

    // Playlist songs
    let mut stmt = conn
        .prepare("SELECT playlist_id, song_id, sort_order FROM playlist_songs ORDER BY playlist_id, sort_order")
        .map_err(|e| e.to_string())?;
    let playlist_songs: Vec<PlaylistSong> = stmt
        .query_map([], |row| {
            Ok(PlaylistSong {
                playlist_id: row.get(0)?,
                song_id: row.get(1)?,
                sort_order: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

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

    // Lyrics
    let mut stmt = conn
        .prepare("SELECT song_id, raw_lrc, source FROM lyrics")
        .map_err(|e| e.to_string())?;
    let lyrics: Vec<Lyric> = stmt
        .query_map([], |row| {
            Ok(Lyric {
                song_id: row.get(0)?,
                raw_lrc: row.get(1)?,
                source: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let backup = BackupFile {
        version: 1,
        app: "scx-music".to_string(),
        exported_at: now,
        data: BackupData {
            songs,
            playlists,
            playlist_songs,
            settings,
            lyrics,
        },
    };

    let json = serde_json::to_string_pretty(&backup).map_err(|e| e.to_string())?;
    fs::write(&save_path, json).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_backup(
    db: tauri::State<'_, Db>,
    file_path: String,
    strategy: String,
) -> Result<ImportResult, String> {
    let content = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    let backup: BackupFileInput = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    if backup.version != 1 {
        return Err(format!("Unsupported backup version: {}", backup.version));
    }

    let data = backup.data;
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

    if strategy == "replace" {
        tx.execute("DELETE FROM playlist_songs", [])
            .map_err(|e| e.to_string())?;
        tx.execute("DELETE FROM lyrics", [])
            .map_err(|e| e.to_string())?;
        tx.execute("DELETE FROM songs", [])
            .map_err(|e| e.to_string())?;
        tx.execute("DELETE FROM playlists", [])
            .map_err(|e| e.to_string())?;
        tx.execute("DELETE FROM settings", [])
            .map_err(|e| e.to_string())?;
    }

    // Songs
    let mut songs_imported = 0usize;
    for s in &data.songs {
        let query = if strategy == "replace" {
            "INSERT INTO songs (id, title, artist, album, duration, duration_secs, quality, file_path, art_gradient) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
        } else {
            "INSERT OR IGNORE INTO songs (id, title, artist, album, duration, duration_secs, quality, file_path, art_gradient) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
        };
        let rows = tx
            .execute(
                query,
                params![s.id, s.title, s.artist, s.album, s.duration, s.duration_secs, s.quality, s.file_path, s.art_gradient],
            )
            .map_err(|e| e.to_string())?;
        songs_imported += rows;
    }

    // Playlists
    let mut playlists_imported = 0usize;
    for p in &data.playlists {
        let query = if strategy == "replace" {
            "INSERT INTO playlists (id, name, sort_order) VALUES (?1, ?2, ?3)"
        } else {
            "INSERT OR IGNORE INTO playlists (id, name, sort_order) VALUES (?1, ?2, ?3)"
        };
        let rows = tx
            .execute(query, params![p.id, p.name, p.sort_order])
            .map_err(|e| e.to_string())?;
        playlists_imported += rows;
    }

    // Playlist songs
    for ps in &data.playlist_songs {
        let query = if strategy == "replace" {
            "INSERT INTO playlist_songs (playlist_id, song_id, sort_order) VALUES (?1, ?2, ?3)"
        } else {
            "INSERT OR IGNORE INTO playlist_songs (playlist_id, song_id, sort_order) VALUES (?1, ?2, ?3)"
        };
        tx.execute(query, params![ps.playlist_id, ps.song_id, ps.sort_order])
            .map_err(|e| e.to_string())?;
    }

    // Lyrics
    let mut lyrics_imported = 0usize;
    for l in &data.lyrics {
        let rows = tx
            .execute(
                "INSERT OR REPLACE INTO lyrics (song_id, raw_lrc, source) VALUES (?1, ?2, ?3)",
                params![l.song_id, l.raw_lrc, l.source],
            )
            .map_err(|e| e.to_string())?;
        lyrics_imported += rows;
    }

    // Settings
    for (key, value) in &data.settings {
        tx.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )
        .map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;

    Ok(ImportResult {
        songs_imported,
        playlists_imported,
        lyrics_imported,
    })
}

#[tauri::command]
pub fn export_settings(
    db: tauri::State<'_, Db>,
    save_path: String,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
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

    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(&save_path, json).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_settings(
    db: tauri::State<'_, Db>,
    file_path: String,
) -> Result<usize, String> {
    let content = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    let settings: HashMap<String, String> =
        serde_json::from_str(&content).map_err(|e| e.to_string())?;

    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

    let mut count = 0usize;
    for (key, value) in &settings {
        tx.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )
        .map_err(|e| e.to_string())?;
        count += 1;
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(count)
}
