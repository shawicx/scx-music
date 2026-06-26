use crate::db::models::{Playlist, Song};
use crate::db::Db;
use crate::error::AppResult;
use rusqlite::params;

#[tauri::command]
pub fn get_playlists(db: tauri::State<'_, Db>) -> AppResult<Vec<Playlist>> {
    let conn = crate::audio::lock_or_recover(&db.0);
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
    let conn = crate::audio::lock_or_recover(&db.0);
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
    let conn = crate::audio::lock_or_recover(&db.0);
    conn.execute("UPDATE playlists SET name = ?1 WHERE id = ?2", params![name, id])?;
    Ok(())
}

#[tauri::command]
pub fn delete_playlist(db: tauri::State<'_, Db>, id: String) -> AppResult<()> {
    let conn = crate::audio::lock_or_recover(&db.0);
    conn.execute("DELETE FROM playlist_songs WHERE playlist_id = ?1", params![id])?;
    conn.execute("DELETE FROM playlists WHERE id = ?1", params![id])?;
    Ok(())
}

#[tauri::command]
pub fn get_playlist_songs(db: tauri::State<'_, Db>, playlist_id: String) -> AppResult<Vec<Song>> {
    let conn = crate::audio::lock_or_recover(&db.0);
    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.title, s.artist, s.album, s.duration, s.duration_secs, s.quality, s.file_path, s.art_gradient, s.genre, s.file_size
             FROM songs s
             INNER JOIN playlist_songs ps ON s.id = ps.song_id
             WHERE ps.playlist_id = ?1
             ORDER BY ps.sort_order",
        )?;
    let songs = stmt
        .query_map(params![playlist_id], Song::from_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(songs)
}

#[tauri::command]
pub fn add_songs_to_playlist(
    db: tauri::State<'_, Db>,
    playlist_id: String,
    song_ids: Vec<String>,
) -> AppResult<()> {
    let conn = crate::audio::lock_or_recover(&db.0);
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

/// 原子替换歌单的全部歌曲：单事务内先 DELETE 再批量 INSERT。
/// 替代前端 clear_playlist + add_songs_to_playlist 两次 IPC，
/// 保证半途崩溃不会留下空歌单（事务原子性）。
#[tauri::command]
pub fn replace_playlist_songs(
    db: tauri::State<'_, Db>,
    playlist_id: String,
    song_ids: Vec<String>,
) -> AppResult<()> {
    let conn = crate::audio::lock_or_recover(&db.0);
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "DELETE FROM playlist_songs WHERE playlist_id = ?1",
        params![playlist_id],
    )?;
    for (i, sid) in song_ids.iter().enumerate() {
        tx.execute(
            "INSERT OR IGNORE INTO playlist_songs (playlist_id, song_id, sort_order) VALUES (?1, ?2, ?3)",
            params![playlist_id, sid, i as i64],
        )?;
    }
    tx.commit()?;
    Ok(())
}

#[tauri::command]
pub fn clear_playlist(db: tauri::State<'_, Db>, playlist_id: String) -> AppResult<()> {
    let conn = crate::audio::lock_or_recover(&db.0);
    conn.execute("DELETE FROM playlist_songs WHERE playlist_id = ?1", params![playlist_id])?;
    Ok(())
}

#[tauri::command]
pub fn remove_song_from_playlist(
    db: tauri::State<'_, Db>,
    playlist_id: String,
    song_id: String,
) -> AppResult<()> {
    let conn = crate::audio::lock_or_recover(&db.0);
    conn.execute(
        "DELETE FROM playlist_songs WHERE playlist_id = ?1 AND song_id = ?2",
        params![playlist_id, song_id],
    )?;
    Ok(())
}
