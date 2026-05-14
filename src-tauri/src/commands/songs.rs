use crate::db::models::Song;
use crate::db::Db;
use rusqlite::params;

#[tauri::command]
pub fn get_all_songs(db: tauri::State<'_, Db>) -> Result<Vec<Song>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, title, artist, album, duration, duration_secs, quality, file_path, art_gradient FROM songs ORDER BY created_at")
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
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(songs)
}

#[tauri::command]
pub fn upsert_songs(db: tauri::State<'_, Db>, songs: Vec<Song>) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    for s in &songs {
        tx.execute(
            "INSERT OR IGNORE INTO songs (id, title, artist, album, duration, duration_secs, quality, file_path, art_gradient)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![s.id, s.title, s.artist, s.album, s.duration, s.duration_secs, s.quality, s.file_path, s.art_gradient],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_songs(db: tauri::State<'_, Db>, ids: Vec<String>) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    for id in &ids {
        tx.execute("DELETE FROM songs WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}
