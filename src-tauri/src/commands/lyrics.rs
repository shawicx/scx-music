use crate::db::Db;
use crate::error::{AppError, AppResult};
use lofty::file::TaggedFileExt;
use lofty::tag::ItemKey;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::OnceLock;

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
) -> AppResult<Option<LyricsResult>> {
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
            ?;
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
    match fetch_lrclib(&title, &artist, duration_secs).await {
        Ok(Some(result)) => {
            let conn = db.0.lock().map_err(|e| e.to_string())?;
            conn.execute(
                "INSERT INTO lyrics (song_id, raw_lrc, source, offset_secs) VALUES (?1, ?2, ?3, ?4)",
                params![song_id, &result.raw_lrc, &result.source, result.offset_secs],
            )
            ?;
            return Ok(Some(result));
        }
        Ok(None) => {
            // LRCLIB 确认无歌词，缓存 source='none'，避免重复请求
        }
        Err(e) => {
            // 网络/服务器错误，不缓存（让用户可重试），向上传播
            return Err(e);
        }
    }

    // 4. No lyrics found — cache the miss
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO lyrics (song_id, raw_lrc, source, offset_secs) VALUES (?1, NULL, 'none', 0.0)",
        params![song_id],
    )
    ?;
    Ok(None)
}

#[tauri::command]
pub async fn refresh_lyrics(
    db: tauri::State<'_, Db>,
    song_id: String,
    title: String,
    artist: String,
    duration_secs: f64,
) -> AppResult<Option<LyricsResult>> {
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

    match fetch_lrclib(&title, &artist, duration_secs).await {
        Ok(Some(result)) => {
            let conn = db.0.lock().map_err(|e| e.to_string())?;
            conn.execute(
                "INSERT OR REPLACE INTO lyrics (song_id, raw_lrc, source, offset_secs) VALUES (?1, ?2, ?3, ?4)",
                params![song_id, &result.raw_lrc, &result.source, existing_offset],
            )
            ?;
            return Ok(Some(LyricsResult {
                raw_lrc: result.raw_lrc,
                source: result.source,
                offset_secs: existing_offset,
            }));
        }
        Ok(None) => {
            // LRCLIB 确认无歌词，缓存 source='none'
        }
        Err(e) => {
            // 网络/服务器错误，不缓存，向上传播让用户可重试
            return Err(e);
        }
    }

    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO lyrics (song_id, raw_lrc, source, offset_secs) VALUES (?1, NULL, 'none', ?2)",
        params![song_id, existing_offset],
    )
    ?;
    Ok(None)
}

#[tauri::command]
pub async fn set_lyric_offset(
    db: tauri::State<'_, Db>,
    song_id: String,
    offset_secs: f64,
) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE lyrics SET offset_secs = ?1 WHERE song_id = ?2",
        params![offset_secs, song_id],
    )
    ?;
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

/// 模块级带超时的 reqwest 客户端，避免每次 fetch_lrclib 调用都重建。
/// 超时 10 秒，覆盖正常 LRCLIB 响应时间，防止网络抖动卡死命令。
static LRCLIB_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn lrclib_client() -> &'static reqwest::Client {
    LRCLIB_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("reqwest client build failed")
    })
}

/// 查询 LRCLIB。
///
/// 返回值语义（错误区分）：
/// - `Ok(Some(result))` — 命中歌词（synced 优先于 plain）
/// - `Ok(None)` — LRCLIB 确认无歌词（HTTP 200 空结果 / 全部结果无歌词字段），
///   调用方应缓存 `source='none'`，避免重复请求
/// - `Err(_)` — 网络/服务器错误（请求失败、非 2xx、JSON 解析失败），
///   调用方应**不缓存**，向上传播让用户可重试
async fn fetch_lrclib(
    title: &str,
    artist: &str,
    duration_secs: f64,
) -> AppResult<Option<LyricsResult>> {
    let query = format!("{} {}", title, artist);
    let client = lrclib_client();

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

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::OperationFailed(format!("LRCLIB 请求失败: {}", e)))?;

    let status = resp.status();
    if !status.is_success() {
        // 非 2xx（含 5xx 服务器错误、429 限流等）视为临时故障，传播让用户重试
        return Err(AppError::OperationFailed(format!(
            "LRCLIB HTTP {}",
            status.as_u16()
        )));
    }

    let results: Vec<LrclibSearchResult> = resp
        .json()
        .await
        .map_err(|e| AppError::OperationFailed(format!("LRCLIB 解析失败: {}", e)))?;
    if results.is_empty() {
        // 搜索无结果 = 确认无歌词，调用方缓存 source='none'
        return Ok(None);
    }

    let best = results.into_iter().min_by(|a, b| {
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
    });

    let Some(best) = best else {
        return Ok(None);
    };

    let raw_lrc = best.synced_lyrics.or(best.plain_lyrics);
    let Some(raw_lrc) = raw_lrc else {
        // 最佳匹配也没有任何歌词字段 → 确认无歌词
        return Ok(None);
    };

    Ok(Some(LyricsResult {
        raw_lrc: Some(raw_lrc),
        source: "lrclib".to_string(),
        offset_secs: 0.0,
    }))
}
