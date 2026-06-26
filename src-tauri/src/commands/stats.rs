use serde::Serialize;
use tauri::State;

use crate::db::Db;
use crate::error::AppResult;
use rusqlite::types::Value as SqlValue;

// ── Library stats ──────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryStats {
    pub total_songs: i64,
    pub total_artists: i64,
    pub total_albums: i64,
    pub total_duration_secs: f64,
    pub total_file_size: i64,
    pub artist_ranking: Vec<ArtistCount>,
    pub album_ranking: Vec<AlbumCount>,
    pub genre_distribution: Vec<GenreCount>,
    pub quality_distribution: Vec<QualityCount>,
    pub duration_distribution: Vec<DurationBucket>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistCount {
    pub artist: String,
    pub song_count: i64,
    pub total_duration_secs: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumCount {
    pub album: String,
    pub artist: String,
    pub song_count: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenreCount {
    pub genre: String,
    pub song_count: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityCount {
    pub quality: String,
    pub song_count: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DurationBucket {
    pub label: String,
    pub song_count: i64,
}

#[tauri::command]
pub fn get_library_stats(db: tauri::State<'_, Db>) -> AppResult<LibraryStats> {
    let conn = crate::audio::lock_or_recover(&db.0);

    let total_songs: i64 = conn
        .query_row("SELECT COUNT(*) FROM songs", [], |r| r.get(0))
        ?;
    let total_artists: i64 = conn
        .query_row("SELECT COUNT(DISTINCT artist) FROM songs WHERE artist NOT IN ('Unknown Artist', '')", [], |r| r.get(0))
        ?;
    let total_albums: i64 = conn
        .query_row("SELECT COUNT(DISTINCT album) FROM songs WHERE album NOT IN ('Unknown Album', '')", [], |r| r.get(0))
        ?;
    let total_duration_secs: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(duration_secs), 0) FROM songs",
            [],
            |r| r.get(0),
        )
        ?;
    let total_file_size: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(file_size), 0) FROM songs",
            [],
            |r| r.get(0),
        )
        ?;

    let mut stmt = conn
        .prepare(
            "SELECT artist, COUNT(*) as cnt, SUM(duration_secs) as dur FROM songs WHERE artist NOT IN ('Unknown Artist', '') GROUP BY artist ORDER BY cnt DESC LIMIT 20",
        )
        ?;
    let artist_ranking = stmt
        .query_map([], |row| {
            Ok(ArtistCount {
                artist: row.get(0)?,
                song_count: row.get(1)?,
                total_duration_secs: row.get(2)?,
            })
        })
        ?
        .collect::<Result<Vec<_>, _>>()
        ?;

    let mut stmt = conn
        .prepare(
            "SELECT album, artist, COUNT(*) as cnt FROM songs WHERE album NOT IN ('Unknown Album', '') AND artist NOT IN ('Unknown Artist', '') GROUP BY album, artist ORDER BY cnt DESC LIMIT 20",
        )
        ?;
    let album_ranking = stmt
        .query_map([], |row| {
            Ok(AlbumCount {
                album: row.get(0)?,
                artist: row.get(1)?,
                song_count: row.get(2)?,
            })
        })
        ?
        .collect::<Result<Vec<_>, _>>()
        ?;

    let mut stmt = conn
        .prepare(
            "SELECT COALESCE(NULLIF(genre, ''), 'Unknown') as genre, COUNT(*) as cnt FROM songs GROUP BY genre ORDER BY cnt DESC",
        )
        ?;
    let genre_distribution = stmt
        .query_map([], |row| {
            Ok(GenreCount {
                genre: row.get(0)?,
                song_count: row.get(1)?,
            })
        })
        ?
        .collect::<Result<Vec<_>, _>>()
        ?;

    let mut stmt = conn
        .prepare(
            "SELECT quality, COUNT(*) as cnt FROM songs GROUP BY quality ORDER BY cnt DESC",
        )
        ?;
    let quality_distribution = stmt
        .query_map([], |row| {
            Ok(QualityCount {
                quality: row.get(0)?,
                song_count: row.get(1)?,
            })
        })
        ?
        .collect::<Result<Vec<_>, _>>()
        ?;

    let mut stmt = conn
        .prepare(
            "SELECT CASE
                WHEN duration_secs < 120 THEN '0-2min'
                WHEN duration_secs < 300 THEN '2-5min'
                WHEN duration_secs < 600 THEN '5-10min'
                ELSE '10min+'
            END as bucket, COUNT(*) as cnt
            FROM songs GROUP BY bucket ORDER BY MIN(duration_secs)",
        )
        ?;
    let duration_distribution = stmt
        .query_map([], |row| {
            Ok(DurationBucket {
                label: row.get(0)?,
                song_count: row.get(1)?,
            })
        })
        ?
        .collect::<Result<Vec<_>, _>>()
        ?;

    Ok(LibraryStats {
        total_songs,
        total_artists,
        total_albums,
        total_duration_secs,
        total_file_size,
        artist_ranking,
        album_ranking,
        genre_distribution,
        quality_distribution,
        duration_distribution,
    })
}

// ── Listening stats ────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListeningOverview {
    pub play_count: i64,
    pub total_duration_secs: f64,
    pub genre_count: i64,
    pub artist_count: i64,
    pub unique_song_count: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopSong {
    pub song_id: String,
    pub title: String,
    pub artist: String,
    pub play_count: i64,
    pub total_duration_secs: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopArtist {
    pub artist: String,
    pub play_count: i64,
    pub total_duration_secs: f64,
    pub song_count: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenreDuration {
    pub genre: String,
    pub duration_secs: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DayDuration {
    pub date: String,
    pub duration_secs: f64,
}

/// 时间过滤的 SQL 片段与对应绑定参数。
/// 支持「滚动窗口」(range) 和「绝对日期」(start/end) 两种模式。
/// 报告 Tab 传 start/end（自然周期），统计 Tab 传 range（滚动窗口）。
fn build_time_filter(
    range: Option<&str>,
    start: Option<&str>,
    end: Option<&str>,
) -> (String, Vec<SqlValue>) {
    // 优先使用绝对日期（报告模式）—— 参数化绑定防 SQL 注入
    if let (Some(s), Some(e)) = (start, end) {
        return (
            "AND ph.played_at >= ? AND ph.played_at < ?".to_string(),
            vec![SqlValue::Text(s.to_string()), SqlValue::Text(e.to_string())],
        );
    }
    // 回退到滚动窗口（现有 StatsView 模式，无需参数）
    match range {
        Some("7d") => ("AND ph.played_at >= datetime('now', '-7 days')".to_string(), vec![]),
        Some("30d") => ("AND ph.played_at >= datetime('now', '-30 days')".to_string(), vec![]),
        _ => (String::new(), vec![]),
    }
}

#[tauri::command]
pub fn stats_listening_overview(
    range: Option<String>,
    start: Option<String>,
    end: Option<String>,
    db: State<'_, Db>,
) -> AppResult<ListeningOverview> {
    let conn = crate::audio::lock_or_recover(&db.0);
    let (filter_sql, filter_params) = build_time_filter(range.as_deref(), start.as_deref(), end.as_deref());

    // 修正：play_count 改为从 play_history 派生，与其他 4 个指标一致地参与时间过滤
    let play_count: i64 = conn.query_row(
        &format!("SELECT COUNT(*) FROM play_history ph WHERE 1=1 {}", filter_sql),
        rusqlite::params_from_iter(filter_params.iter()),
        |r| r.get(0),
    ).unwrap_or(0);

    let total_duration_secs: f64 = conn.query_row(
        &format!("SELECT COALESCE(SUM(ph.duration_secs), 0) FROM play_history ph WHERE 1=1 {}", filter_sql),
        rusqlite::params_from_iter(filter_params.iter()),
        |r| r.get(0),
    ).unwrap_or(0.0);

    let genre_count: i64 = conn.query_row(
        &format!("SELECT COUNT(DISTINCT s.genre) FROM play_history ph JOIN songs s ON s.id = ph.song_id WHERE s.genre != '' {}", filter_sql),
        rusqlite::params_from_iter(filter_params.iter()),
        |r| r.get(0),
    ).unwrap_or(0);

    let artist_count: i64 = conn.query_row(
        &format!("SELECT COUNT(DISTINCT s.artist) FROM play_history ph JOIN songs s ON s.id = ph.song_id WHERE s.artist NOT IN ('Unknown Artist', '') {}", filter_sql),
        rusqlite::params_from_iter(filter_params.iter()),
        |r| r.get(0),
    ).unwrap_or(0);

    let unique_song_count: i64 = conn.query_row(
        &format!("SELECT COUNT(DISTINCT ph.song_id) FROM play_history ph WHERE 1=1 {}", filter_sql),
        rusqlite::params_from_iter(filter_params.iter()),
        |r| r.get(0),
    ).unwrap_or(0);

    Ok(ListeningOverview { play_count, total_duration_secs, genre_count, artist_count, unique_song_count })
}

#[tauri::command]
pub fn stats_top_songs(
    range: Option<String>,
    limit: Option<i64>,
    db: State<'_, Db>,
) -> AppResult<Vec<TopSong>> {
    let conn = crate::audio::lock_or_recover(&db.0);
    let limit = limit.unwrap_or(10);
    let (filter_sql, mut filter_params) = build_time_filter(range.as_deref(), None, None);

    let mut stmt = conn.prepare(&format!(
        "SELECT s.id, s.title, s.artist, COUNT(*) as play_count, COALESCE(SUM(ph.duration_secs), 0) as dur
         FROM play_history ph JOIN songs s ON s.id = ph.song_id WHERE 1=1 {}
         GROUP BY s.id ORDER BY dur DESC LIMIT ?", filter_sql
    ))?;

    filter_params.push(SqlValue::Integer(limit));
    let rows: Vec<TopSong> = stmt.query_map(rusqlite::params_from_iter(filter_params.iter()), |row| {
        Ok(TopSong {
            song_id: row.get(0)?, title: row.get(1)?, artist: row.get(2)?,
            play_count: row.get(3)?, total_duration_secs: row.get(4)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

#[tauri::command]
pub fn stats_top_artists(
    range: Option<String>,
    limit: Option<i64>,
    db: State<'_, Db>,
) -> AppResult<Vec<TopArtist>> {
    let conn = crate::audio::lock_or_recover(&db.0);
    let limit = limit.unwrap_or(10);
    let (filter_sql, mut filter_params) = build_time_filter(range.as_deref(), None, None);

    let mut stmt = conn.prepare(&format!(
        "SELECT s.artist, COUNT(*) as cnt, COALESCE(SUM(ph.duration_secs), 0) as dur, COUNT(DISTINCT s.id) as songs
         FROM play_history ph JOIN songs s ON s.id = ph.song_id WHERE s.artist NOT IN ('Unknown Artist', '') {}
         GROUP BY s.artist ORDER BY dur DESC LIMIT ?", filter_sql
    ))?;

    filter_params.push(SqlValue::Integer(limit));
    let rows: Vec<TopArtist> = stmt.query_map(rusqlite::params_from_iter(filter_params.iter()), |row| {
        Ok(TopArtist {
            artist: row.get(0)?, play_count: row.get(1)?,
            total_duration_secs: row.get(2)?, song_count: row.get(3)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

#[tauri::command]
pub fn stats_genre_distribution(
    range: Option<String>,
    db: State<'_, Db>,
) -> AppResult<Vec<GenreDuration>> {
    let conn = crate::audio::lock_or_recover(&db.0);
    let (filter_sql, filter_params) = build_time_filter(range.as_deref(), None, None);

    let mut stmt = conn.prepare(&format!(
        "SELECT COALESCE(NULLIF(s.genre, ''), 'Unknown') as genre, SUM(ph.duration_secs) as dur
         FROM play_history ph JOIN songs s ON s.id = ph.song_id WHERE 1=1 {}
         GROUP BY genre ORDER BY dur DESC", filter_sql
    ))?;

    let rows: Vec<GenreDuration> = stmt.query_map(rusqlite::params_from_iter(filter_params.iter()), |row| {
        Ok(GenreDuration { genre: row.get(0)?, duration_secs: row.get(1)? })
    })?.collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

#[tauri::command]
pub fn stats_trend(
    range: Option<String>,
    db: State<'_, Db>,
) -> AppResult<Vec<DayDuration>> {
    let conn = crate::audio::lock_or_recover(&db.0);
    let (filter_sql, filter_params) = build_time_filter(range.as_deref(), None, None);

    let mut stmt = conn.prepare(&format!(
        "SELECT DATE(ph.played_at) as day, SUM(ph.duration_secs) as dur
         FROM play_history ph WHERE 1=1 {} GROUP BY day ORDER BY day", filter_sql
    ))?;

    let rows: Vec<DayDuration> = stmt.query_map(rusqlite::params_from_iter(filter_params.iter()), |row| {
        Ok(DayDuration { date: row.get(0)?, duration_secs: row.get(1)? })
    })?.collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

#[tauri::command]
pub fn stats_heatmap(db: State<'_, Db>) -> AppResult<Vec<DayDuration>> {
    let conn = crate::audio::lock_or_recover(&db.0);

    let mut stmt = conn.prepare(
        "SELECT DATE(ph.played_at) as day, SUM(ph.duration_secs) as dur
         FROM play_history ph WHERE ph.played_at >= datetime('now', '-365 days')
         GROUP BY day ORDER BY day"
    )?;

    let rows: Vec<DayDuration> = stmt.query_map([], |row| {
        Ok(DayDuration { date: row.get(0)?, duration_secs: row.get(1)? })
    })?.collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HourDuration {
    pub hour: i64,
    pub duration_secs: f64,
}

#[tauri::command]
pub fn stats_hourly_distribution(
    start: String,
    end: String,
    db: State<'_, Db>,
) -> AppResult<Vec<HourDuration>> {
    let conn = crate::audio::lock_or_recover(&db.0);

    let mut stmt = conn.prepare(
        "SELECT CAST(strftime('%H', ph.played_at, 'localtime') AS INTEGER) as hour,
                SUM(ph.duration_secs) as dur
         FROM play_history ph
         WHERE ph.played_at >= ?1 AND ph.played_at < ?2
         GROUP BY hour ORDER BY hour"
    )?;

    let rows: Vec<HourDuration> = stmt
        .query_map(rusqlite::params![start, end], |row| {
            Ok(HourDuration {
                hour: row.get(0)?,
                duration_secs: row.get(1)?,
            })
        })
        ?
        .collect::<Result<Vec<_>, _>>()
        ?;
    Ok(rows)
}

// ── Dashboard 聚合（替代前端 6 次扇出）──────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsDashboard {
    pub overview: ListeningOverview,
    pub top_songs: Vec<TopSong>,
    pub top_artists: Vec<TopArtist>,
    pub genre_distribution: Vec<GenreDuration>,
    pub trend: Vec<DayDuration>,
    pub heatmap: Vec<DayDuration>,
}

/// 单次 IPC 返回统计仪表盘全部数据，替代前端 Promise.all 发 6 个命令。
/// 一次锁、一次 prepare 各 SQL，消除 6 次往返与重复锁竞争。
///
/// 参数语义与原 6 个命令一致：
/// - `range` / `start` / `end`：时间过滤（报告模式用 start/end，统计模式用 range）
/// - `top_limit`：top_songs / top_artists 的 LIMIT，默认 10
#[tauri::command]
pub fn stats_dashboard(
    range: Option<String>,
    start: Option<String>,
    end: Option<String>,
    top_limit: Option<i64>,
    db: State<'_, Db>,
) -> AppResult<StatsDashboard> {
    let conn = crate::audio::lock_or_recover(&db.0);
    let limit = top_limit.unwrap_or(10);
    let (filter_sql, filter_params) =
        build_time_filter(range.as_deref(), start.as_deref(), end.as_deref());

    // ── overview（原 stats_listening_overview 的 5 个 query_row）──
    let play_count: i64 = conn
        .query_row(
            &format!("SELECT COUNT(*) FROM play_history ph WHERE 1=1 {}", filter_sql),
            rusqlite::params_from_iter(filter_params.iter()),
            |r| r.get(0),
        )
        .unwrap_or(0);

    let total_duration_secs: f64 = conn
        .query_row(
            &format!(
                "SELECT COALESCE(SUM(ph.duration_secs), 0) FROM play_history ph WHERE 1=1 {}",
                filter_sql
            ),
            rusqlite::params_from_iter(filter_params.iter()),
            |r| r.get(0),
        )
        .unwrap_or(0.0);

    let genre_count: i64 = conn
        .query_row(
            &format!(
                "SELECT COUNT(DISTINCT s.genre) FROM play_history ph JOIN songs s ON s.id = ph.song_id WHERE s.genre != '' {}",
                filter_sql
            ),
            rusqlite::params_from_iter(filter_params.iter()),
            |r| r.get(0),
        )
        .unwrap_or(0);

    let artist_count: i64 = conn
        .query_row(
            &format!(
                "SELECT COUNT(DISTINCT s.artist) FROM play_history ph JOIN songs s ON s.id = ph.song_id WHERE s.artist NOT IN ('Unknown Artist', '') {}",
                filter_sql
            ),
            rusqlite::params_from_iter(filter_params.iter()),
            |r| r.get(0),
        )
        .unwrap_or(0);

    let unique_song_count: i64 = conn
        .query_row(
            &format!(
                "SELECT COUNT(DISTINCT ph.song_id) FROM play_history ph WHERE 1=1 {}",
                filter_sql
            ),
            rusqlite::params_from_iter(filter_params.iter()),
            |r| r.get(0),
        )
        .unwrap_or(0);

    let overview = ListeningOverview {
        play_count,
        total_duration_secs,
        genre_count,
        artist_count,
        unique_song_count,
    };

    // ── top_songs ──
    let mut stmt = conn.prepare(&format!(
        "SELECT s.id, s.title, s.artist, COUNT(*) as play_count, COALESCE(SUM(ph.duration_secs), 0) as dur
         FROM play_history ph JOIN songs s ON s.id = ph.song_id WHERE 1=1 {}
         GROUP BY s.id ORDER BY dur DESC LIMIT ?", filter_sql
    ))?;
    let mut top_params = filter_params.clone();
    top_params.push(SqlValue::Integer(limit));
    let top_songs: Vec<TopSong> = stmt
        .query_map(rusqlite::params_from_iter(top_params.iter()), |row| {
            Ok(TopSong {
                song_id: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                play_count: row.get(3)?,
                total_duration_secs: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // ── top_artists ──
    let mut stmt = conn.prepare(&format!(
        "SELECT s.artist, COUNT(*) as cnt, COALESCE(SUM(ph.duration_secs), 0) as dur, COUNT(DISTINCT s.id) as songs
         FROM play_history ph JOIN songs s ON s.id = ph.song_id WHERE s.artist NOT IN ('Unknown Artist', '') {}
         GROUP BY s.artist ORDER BY dur DESC LIMIT ?", filter_sql
    ))?;
    let top_artists: Vec<TopArtist> = stmt
        .query_map(rusqlite::params_from_iter(top_params.iter()), |row| {
            Ok(TopArtist {
                artist: row.get(0)?,
                play_count: row.get(1)?,
                total_duration_secs: row.get(2)?,
                song_count: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // ── genre_distribution ──
    let mut stmt = conn.prepare(&format!(
        "SELECT COALESCE(NULLIF(s.genre, ''), 'Unknown') as genre, SUM(ph.duration_secs) as dur
         FROM play_history ph JOIN songs s ON s.id = ph.song_id WHERE 1=1 {}
         GROUP BY genre ORDER BY dur DESC", filter_sql
    ))?;
    let genre_distribution: Vec<GenreDuration> = stmt
        .query_map(rusqlite::params_from_iter(filter_params.iter()), |row| {
            Ok(GenreDuration {
                genre: row.get(0)?,
                duration_secs: row.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // ── trend ──
    let mut stmt = conn.prepare(&format!(
        "SELECT DATE(ph.played_at) as day, SUM(ph.duration_secs) as dur
         FROM play_history ph WHERE 1=1 {} GROUP BY day ORDER BY day", filter_sql
    ))?;
    let trend: Vec<DayDuration> = stmt
        .query_map(rusqlite::params_from_iter(filter_params.iter()), |row| {
            Ok(DayDuration {
                date: row.get(0)?,
                duration_secs: row.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // ── heatmap（固定 365 天，不受 range/start/end 影响，与原 stats_heatmap 一致）──
    let mut stmt = conn.prepare(
        "SELECT DATE(ph.played_at) as day, SUM(ph.duration_secs) as dur
         FROM play_history ph WHERE ph.played_at >= datetime('now', '-365 days')
         GROUP BY day ORDER BY day",
    )?;
    let heatmap: Vec<DayDuration> = stmt
        .query_map([], |row| {
            Ok(DayDuration {
                date: row.get(0)?,
                duration_secs: row.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(StatsDashboard {
        overview,
        top_songs,
        top_artists,
        genre_distribution,
        trend,
        heatmap,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_dates_bind_as_parameters() {
        let (sql, params) =
            build_time_filter(None, Some("2026-01-01"), Some("2026-02-01"));
        assert!(sql.contains("AND ph.played_at >= ? AND ph.played_at < ?"), "sql={sql}");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn range_7d_uses_sqlite_datetime_with_no_params() {
        let (sql, params) = build_time_filter(Some("7d"), None, None);
        assert!(sql.contains("datetime('now', '-7 days')"), "sql={sql}");
        assert!(params.is_empty());
    }

    #[test]
    fn no_filter_returns_empty() {
        let (sql, params) = build_time_filter(None, None, None);
        assert!(sql.is_empty(), "sql={sql}");
        assert!(params.is_empty());
    }
}
