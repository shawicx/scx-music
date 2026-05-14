use crate::db::models::{Playlist, Song};
use crate::db::Db;
use rusqlite::params;

#[tauri::command]
pub fn get_playlists(db: tauri::State<'_, Db>) -> Result<Vec<Playlist>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
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
    Ok(playlists)
}

#[tauri::command]
pub fn create_playlist(db: tauri::State<'_, Db>, name: String) -> Result<Playlist, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let id = format!("pl-{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis());
    let sort_order: i64 = conn
        .query_row("SELECT COALESCE(MAX(sort_order), -1) + 1 FROM playlists", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO playlists (id, name, sort_order) VALUES (?1, ?2, ?3)",
        params![id, name, sort_order],
    )
    .map_err(|e| e.to_string())?;
    Ok(Playlist { id, name, sort_order })
}

#[tauri::command]
pub fn rename_playlist(db: tauri::State<'_, Db>, id: String, name: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("UPDATE playlists SET name = ?1 WHERE id = ?2", params![name, id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_playlist(db: tauri::State<'_, Db>, id: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM playlist_songs WHERE playlist_id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM playlists WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_playlist_songs(db: tauri::State<'_, Db>, playlist_id: String) -> Result<Vec<Song>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.title, s.artist, s.album, s.duration, s.duration_secs, s.quality, s.file_path, s.art_gradient
             FROM songs s
             INNER JOIN playlist_songs ps ON s.id = ps.song_id
             WHERE ps.playlist_id = ?1
             ORDER BY ps.sort_order",
        )
        .map_err(|e| e.to_string())?;
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
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(songs)
}

#[tauri::command]
pub fn add_songs_to_playlist(
    db: tauri::State<'_, Db>,
    playlist_id: String,
    song_ids: Vec<String>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    for sid in &song_ids {
        let sort_order: i64 = tx
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM playlist_songs WHERE playlist_id = ?1",
                params![playlist_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT OR IGNORE INTO playlist_songs (playlist_id, song_id, sort_order) VALUES (?1, ?2, ?3)",
            params![playlist_id, sid, sort_order],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn remove_song_from_playlist(
    db: tauri::State<'_, Db>,
    playlist_id: String,
    song_id: String,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM playlist_songs WHERE playlist_id = ?1 AND song_id = ?2",
        params![playlist_id, song_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
