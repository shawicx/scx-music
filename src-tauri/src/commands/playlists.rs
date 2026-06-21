use crate::db::models::{Playlist, Song};
use crate::db::Db;
use crate::error::AppResult;
use rusqlite::params;

#[tauri::command]
pub fn get_playlists(db: tauri::State<'_, Db>) -> AppResult<Vec<Playlist>> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
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
    Ok(playlists)
}

#[tauri::command]
pub fn create_playlist(db: tauri::State<'_, Db>, name: String) -> AppResult<Playlist> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let id = format!("pl-{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis());
    let sort_order: i64 = conn
        .query_row("SELECT COALESCE(MAX(sort_order), -1) + 1 FROM playlists", [], |row| row.get(0))?;
    conn.execute(
        "INSERT INTO playlists (id, name, sort_order) VALUES (?1, ?2, ?3)",
        params![id, name, sort_order],
    )?;
    Ok(Playlist { id, name, sort_order })
}

#[tauri::command]
pub fn rename_playlist(db: tauri::State<'_, Db>, id: String, name: String) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("UPDATE playlists SET name = ?1 WHERE id = ?2", params![name, id])?;
    Ok(())
}

#[tauri::command]
pub fn delete_playlist(db: tauri::State<'_, Db>, id: String) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM playlist_songs WHERE playlist_id = ?1", params![id])?;
    conn.execute("DELETE FROM playlists WHERE id = ?1", params![id])?;
    Ok(())
}

#[tauri::command]
pub fn get_playlist_songs(db: tauri::State<'_, Db>, playlist_id: String) -> AppResult<Vec<Song>> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.title, s.artist, s.album, s.duration, s.duration_secs, s.quality, s.file_path, s.art_gradient, s.genre, s.file_size
             FROM songs s
             INNER JOIN playlist_songs ps ON s.id = ps.song_id
             WHERE ps.playlist_id = ?1
             ORDER BY ps.sort_order",
        )?;
    let songs = stmt
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
                genre: row.get(9)?,
                file_size: row.get(10)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(songs)
}

#[tauri::command]
pub fn add_songs_to_playlist(
    db: tauri::State<'_, Db>,
    playlist_id: String,
    song_ids: Vec<String>,
) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let tx = conn.unchecked_transaction()?;
    for sid in &song_ids {
        let sort_order: i64 = tx
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM playlist_songs WHERE playlist_id = ?1",
                params![playlist_id],
                |row| row.get(0),
            )?;
        tx.execute(
            "INSERT OR IGNORE INTO playlist_songs (playlist_id, song_id, sort_order) VALUES (?1, ?2, ?3)",
            params![playlist_id, sid, sort_order],
        )?;
    }
    tx.commit()?;
    Ok(())
}

#[tauri::command]
pub fn clear_playlist(db: tauri::State<'_, Db>, playlist_id: String) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM playlist_songs WHERE playlist_id = ?1", params![playlist_id])?;
    Ok(())
}

#[tauri::command]
pub fn remove_song_from_playlist(
    db: tauri::State<'_, Db>,
    playlist_id: String,
    song_id: String,
) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM playlist_songs WHERE playlist_id = ?1 AND song_id = ?2",
        params![playlist_id, song_id],
    )?;
    Ok(())
}
