use crate::db::models::Song;
use crate::db::Db;
use crate::error::{AppError, AppResult};
use lofty::config::WriteOptions;
use lofty::file::{AudioFile, TaggedFileExt};
use rusqlite::params;

#[tauri::command]
pub fn get_all_songs(db: tauri::State<'_, Db>) -> AppResult<Vec<Song>> {
    let conn = crate::audio::lock_or_recover(&db.0);
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
    Ok(songs)
}

#[tauri::command]
pub fn upsert_songs(db: tauri::State<'_, Db>, songs: Vec<Song>) -> AppResult<Vec<String>> {
    let conn = crate::audio::lock_or_recover(&db.0);
    let tx = conn.unchecked_transaction()?;
    for s in &songs {
        tx.execute(
            "INSERT INTO songs (id, title, artist, album, duration, duration_secs, quality, file_path, art_gradient, genre, file_size)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(file_path) DO UPDATE SET
               title = excluded.title,
               artist = excluded.artist,
               album = excluded.album,
               duration = excluded.duration,
               duration_secs = excluded.duration_secs,
               quality = excluded.quality,
               art_gradient = excluded.art_gradient,
               genre = excluded.genre,
               file_size = excluded.file_size",
            params![s.id, s.title, s.artist, s.album, s.duration, s.duration_secs, s.quality, s.file_path, s.art_gradient, s.genre, s.file_size],
        )?;
    }
    let mut ids = Vec::with_capacity(songs.len());
    for s in &songs {
        let id: String = tx
            .query_row("SELECT id FROM songs WHERE file_path = ?1", params![s.file_path], |row| row.get(0))?;
        ids.push(id);
    }
    tx.commit()?;
    Ok(ids)
}

#[tauri::command]
pub fn delete_songs(db: tauri::State<'_, Db>, ids: Vec<String>) -> AppResult<()> {
    let conn = crate::audio::lock_or_recover(&db.0);
    let tx = conn.unchecked_transaction()?;
    for id in &ids {
        tx.execute("DELETE FROM songs WHERE id = ?1", params![id])?;
    }
    tx.commit()?;
    Ok(())
}

#[tauri::command]
pub fn rename_song(
    song_id: String,
    new_title: String,
    new_artist: Option<String>,
    new_album: Option<String>,
    db: tauri::State<'_, Db>,
) -> AppResult<Song> {
    // 1. Query current song (release lock immediately)
    let (old_file_path, old_artist, old_album): (String, String, String) = {
        let conn = crate::audio::lock_or_recover(&db.0);
        conn.query_row(
            "SELECT file_path, artist, album FROM songs WHERE id = ?1",
            params![song_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|e| format!("Song not found: {}", e))?
    };

    // 2. Validate file exists and is writable
    let old_path = std::path::Path::new(&old_file_path);
    if !old_path.exists() {
        return Err(AppError::FileOperation("File not found on disk".to_string()));
    }

    let parent = old_path.parent().ok_or_else(|| AppError::InvalidArgument("Cannot determine parent directory".to_string()))?;
    let metadata = std::fs::metadata(&old_file_path).map_err(|e| format!("Cannot read file: {}", e))?;
    if metadata.permissions().readonly() {
        return Err(AppError::FileOperation("File is read-only".to_string()));
    }

    // 3. Write metadata tags via Lofty — 不再静默跳过失败
    //    （用户改了标题，期望标题元数据被写入；解析失败应显式报错）
    let artist = new_artist.unwrap_or(old_artist);
    let album = new_album.unwrap_or(old_album);

    let mut tagged = lofty::read_from_path(&old_file_path)
        .map_err(|e| format!("Failed to read metadata: {}", e))?;
    if let Some(tag) = tagged.primary_tag_mut() {
        tag.insert_text(lofty::tag::ItemKey::TrackTitle, new_title.clone());
        tag.insert_text(lofty::tag::ItemKey::TrackArtist, artist.clone());
        tag.insert_text(lofty::tag::ItemKey::AlbumTitle, album.clone());
    }
    tagged
        .save_to_path(&old_file_path, WriteOptions::default())
        .map_err(|e| format!("Failed to write metadata: {}", e))?;

    // 4. Build new filename, resolve conflicts (TOCTOU accepted; rename fallback below)
    let extension = old_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_string();

    let new_filename = if extension.is_empty() {
        sanitize_filename(&new_title)
    } else {
        format!("{}.{}", sanitize_filename(&new_title), extension)
    };

    let mut new_path = parent.join(&new_filename);
    let mut counter = 2u32;
    while new_path.exists() && new_path != old_path {
        let conflict_name = if extension.is_empty() {
            format!("{} ({})", sanitize_filename(&new_title), counter)
        } else {
            format!("{} ({}).{}", sanitize_filename(&new_title), counter, extension)
        };
        new_path = parent.join(&conflict_name);
        counter += 1;
    }

    // 5. Rename file on disk
    let renamed = new_path != old_path;
    if renamed {
        std::fs::rename(&old_file_path, new_path.to_string_lossy().to_string())
            .map_err(|e| format!("Failed to rename file: {}", e))?;
    }

    let new_file_path = new_path.to_string_lossy().to_string();

    // 6. Update database inside a transaction (re-acquire lock).
    //    若事务失败且文件已被重命名，尽力回滚文件名，避免文件与 DB 不一致。
    let db_result: AppResult<()> = {
        let mut conn = crate::audio::lock_or_recover(&db.0);
        let tx = conn.transaction()?;
        tx.execute(
            "UPDATE songs SET title = ?1, artist = ?2, album = ?3, file_path = ?4 WHERE id = ?5",
            params![new_title, artist, album, new_file_path, song_id],
        )?;
        tx.commit()?;
        Ok(())
    };
    if let Err(e) = db_result {
        if renamed {
            let rollback = std::fs::rename(&new_file_path, &old_file_path);
            if let Err(rollback_err) = rollback {
                eprintln!(
                    "[rename_song] rollback failed: {} → {}: {}",
                    new_path.display(),
                    old_path.display(),
                    rollback_err
                );
            }
        }
        return Err(e);
    }

    // 7. Return updated song
    let conn = crate::audio::lock_or_recover(&db.0);
    let song = conn
        .query_row(
            "SELECT id, title, artist, album, duration, duration_secs, quality, file_path, art_gradient, genre, file_size FROM songs WHERE id = ?1",
            params![song_id],
            |row| {
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
            },
        )?;

    Ok(song)
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}
