use crate::db::Db;
use lofty::file::TaggedFileExt;
use lofty::tag::ItemKey;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LrclibSearchResult {
    id: i64,
    track_name: String,
    artist_name: String,
    duration: Option<f64>,
    synced_lyrics: Option<String>,
    plain_lyrics: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricsResult {
    pub raw_lrc: Option<String>,
    pub source: String,
    pub offset_secs: f64,
}

#[tauri::command]
pub async fn get_lyrics(
    db: tauri::State<'_, Db>,
    song_id: String,
    file_path: String,
    title: String,
    artist: String,
    duration_secs: f64,
) -> Result<Option<LyricsResult>, String> {
    // 1. Check SQLite cache + 2. Try embedded lyrics (sync, no .await)
    let early_result = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;

        let cached: Option<(Option<String>, String, f64)> = conn
            .query_row(
                "SELECT raw_lrc, source, offset_secs FROM lyrics WHERE song_id = ?1",
                params![song_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .ok();

        if let Some((raw_lrc, source, offset_secs)) = cached {
            Some(Ok(Some(LyricsResult { raw_lrc, source, offset_secs })))
        } else if let Some(raw_lrc) = extract_embedded_lyrics(&file_path) {
            let source = "embedded".to_string();
            conn.execute(
                "INSERT INTO lyrics (song_id, raw_lrc, source, offset_secs) VALUES (?1, ?2, ?3, 0.0)",
                params![song_id, raw_lrc, source],
            )
            .map_err(|e| e.to_string())?;
            Some(Ok(Some(LyricsResult {
                raw_lrc: Some(raw_lrc),
                source,
                offset_secs: 0.0,
            })))
        } else {
            None
        }
    };

    if let Some(result) = early_result {
        return result;
    }

    // 3. Try LRCLIB online (conn is dropped, safe to .await)
    if let Some(result) = fetch_lrclib(&title, &artist, duration_secs).await {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO lyrics (song_id, raw_lrc, source, offset_secs) VALUES (?1, ?2, ?3, ?4)",
            params![song_id, &result.raw_lrc, &result.source, result.offset_secs],
        )
        .map_err(|e| e.to_string())?;
        return Ok(Some(result));
    }

    // 4. No lyrics found — cache the miss
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO lyrics (song_id, raw_lrc, source, offset_secs) VALUES (?1, NULL, 'none', 0.0)",
        params![song_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(None)
}

#[tauri::command]
pub async fn refresh_lyrics(
    db: tauri::State<'_, Db>,
    song_id: String,
    title: String,
    artist: String,
    duration_secs: f64,
) -> Result<Option<LyricsResult>, String> {
    // Preserve existing offset before overwrite
    let existing_offset: f64 = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT offset_secs FROM lyrics WHERE song_id = ?1",
            params![song_id],
            |row| row.get(0),
        )
        .unwrap_or(0.0)
    };

    if let Some(result) = fetch_lrclib(&title, &artist, duration_secs).await {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO lyrics (song_id, raw_lrc, source, offset_secs) VALUES (?1, ?2, ?3, ?4)",
            params![song_id, &result.raw_lrc, &result.source, existing_offset],
        )
        .map_err(|e| e.to_string())?;
        return Ok(Some(LyricsResult {
            raw_lrc: result.raw_lrc,
            source: result.source,
            offset_secs: existing_offset,
        }));
    }

    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO lyrics (song_id, raw_lrc, source, offset_secs) VALUES (?1, NULL, 'none', ?2)",
        params![song_id, existing_offset],
    )
    .map_err(|e| e.to_string())?;
    Ok(None)
}

#[tauri::command]
pub async fn set_lyric_offset(
    db: tauri::State<'_, Db>,
    song_id: String,
    offset_secs: f64,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE lyrics SET offset_secs = ?1 WHERE song_id = ?2",
        params![offset_secs, song_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn extract_embedded_lyrics(file_path: &str) -> Option<String> {
    let path = Path::new(file_path);
    let tagged = lofty::read_from_path(path).ok()?;
    let tag = tagged.primary_tag()?;

    if let Some(lyrics) = tag.get_string(&ItemKey::Lyrics) {
        let text = lyrics.to_string();
        if !text.trim().is_empty() {
            return Some(text);
        }
    }

    None
}

async fn fetch_lrclib(title: &str, artist: &str, duration_secs: f64) -> Option<LyricsResult> {
    let query = format!("{} {}", title, artist);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .ok()?;

    let url = if duration_secs > 0.0 {
        format!(
            "https://lrclib.net/api/search?q={}&duration={}",
            urlencoding::encode(&query),
            duration_secs.round()
        )
    } else {
        format!(
            "https://lrclib.net/api/search?q={}",
            urlencoding::encode(&query)
        )
    };

    let resp = client.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }

    let results: Vec<LrclibSearchResult> = resp.json().await.ok()?;
    if results.is_empty() {
        return None;
    }

    let best = results
        .into_iter()
        .min_by(|a, b| {
            let a_score = match (a.duration, duration_secs > 0.0) {
                (Some(d), true) => (d - duration_secs).abs(),
                _ => f64::MAX,
            };
            let b_score = match (b.duration, duration_secs > 0.0) {
                (Some(d), true) => (d - duration_secs).abs(),
                _ => f64::MAX,
            };
            match (a.synced_lyrics.is_some(), b.synced_lyrics.is_some()) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a_score
                    .partial_cmp(&b_score)
                    .unwrap_or(std::cmp::Ordering::Equal),
            }
        })?;

    let raw_lrc = best.synced_lyrics.or(best.plain_lyrics);
    let source = "lrclib".to_string();
    Some(LyricsResult {
        raw_lrc,
        source,
        offset_secs: 0.0,
    })
}
