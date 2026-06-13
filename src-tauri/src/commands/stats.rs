use serde::Serialize;
use tauri::State;

use crate::db::Db;

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
pub fn get_library_stats(db: tauri::State<'_, Db>) -> Result<LibraryStats, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let total_songs: i64 = conn
        .query_row("SELECT COUNT(*) FROM songs", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let total_artists: i64 = conn
        .query_row("SELECT COUNT(DISTINCT artist) FROM songs WHERE artist NOT IN ('Unknown Artist', '')", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let total_albums: i64 = conn
        .query_row("SELECT COUNT(DISTINCT album) FROM songs WHERE album NOT IN ('Unknown Album', '')", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let total_duration_secs: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(duration_secs), 0) FROM songs",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    let total_file_size: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(file_size), 0) FROM songs",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT artist, COUNT(*) as cnt, SUM(duration_secs) as dur FROM songs WHERE artist NOT IN ('Unknown Artist', '') GROUP BY artist ORDER BY cnt DESC LIMIT 20",
        )
        .map_err(|e| e.to_string())?;
    let artist_ranking = stmt
        .query_map([], |row| {
            Ok(ArtistCount {
                artist: row.get(0)?,
                song_count: row.get(1)?,
                total_duration_secs: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT album, artist, COUNT(*) as cnt FROM songs WHERE album NOT IN ('Unknown Album', '') AND artist NOT IN ('Unknown Artist', '') GROUP BY album, artist ORDER BY cnt DESC LIMIT 20",
        )
        .map_err(|e| e.to_string())?;
    let album_ranking = stmt
        .query_map([], |row| {
            Ok(AlbumCount {
                album: row.get(0)?,
                artist: row.get(1)?,
                song_count: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT COALESCE(NULLIF(genre, ''), 'Unknown') as genre, COUNT(*) as cnt FROM songs GROUP BY genre ORDER BY cnt DESC",
        )
        .map_err(|e| e.to_string())?;
    let genre_distribution = stmt
        .query_map([], |row| {
            Ok(GenreCount {
                genre: row.get(0)?,
                song_count: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT quality, COUNT(*) as cnt FROM songs GROUP BY quality ORDER BY cnt DESC",
        )
        .map_err(|e| e.to_string())?;
    let quality_distribution = stmt
        .query_map([], |row| {
            Ok(QualityCount {
                quality: row.get(0)?,
                song_count: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

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
        .map_err(|e| e.to_string())?;
    let duration_distribution = stmt
        .query_map([], |row| {
            Ok(DurationBucket {
                label: row.get(0)?,
                song_count: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

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

fn range_filter(range: &str) -> String {
    match range {
        "7d" => "AND ph.played_at >= datetime('now', '-7 days')".to_string(),
        "30d" => "AND ph.played_at >= datetime('now', '-30 days')".to_string(),
        _ => String::new(), // "all" = no filter
    }
}

#[tauri::command]
pub fn stats_listening_overview(range: String, db: State<'_, Db>) -> Result<ListeningOverview, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let filter = range_filter(&range);

    let play_count: i64 = conn.query_row(
        "SELECT COALESCE(SUM(play_count), 0) FROM songs",
        [], |r| r.get(0),
    ).unwrap_or(0);

    let total_duration_secs: f64 = conn.query_row(
        &format!("SELECT COALESCE(SUM(ph.duration_secs), 0) FROM play_history ph WHERE 1=1 {}", filter),
        [], |r| r.get(0),
    ).unwrap_or(0.0);

    let genre_count: i64 = conn.query_row(
        &format!("SELECT COUNT(DISTINCT s.genre) FROM play_history ph JOIN songs s ON s.id = ph.song_id WHERE s.genre != '' {}", filter),
        [], |r| r.get(0),
    ).unwrap_or(0);

    let artist_count: i64 = conn.query_row(
        &format!("SELECT COUNT(DISTINCT s.artist) FROM play_history ph JOIN songs s ON s.id = ph.song_id WHERE s.artist NOT IN ('Unknown Artist', '') {}", filter),
        [], |r| r.get(0),
    ).unwrap_or(0);

    Ok(ListeningOverview { play_count, total_duration_secs, genre_count, artist_count })
}

#[tauri::command]
pub fn stats_top_songs(range: String, limit: Option<i64>, db: State<'_, Db>) -> Result<Vec<TopSong>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let limit = limit.unwrap_or(10);
    let filter = range_filter(&range);

    let mut stmt = conn.prepare(&format!(
        "SELECT s.id, s.title, s.artist, s.play_count, COALESCE(SUM(ph.duration_secs), 0) as dur
         FROM play_history ph JOIN songs s ON s.id = ph.song_id WHERE 1=1 {}
         GROUP BY s.id ORDER BY dur DESC LIMIT ?", filter
    )).map_err(|e| e.to_string())?;

    let rows: Vec<TopSong> = stmt.query_map(rusqlite::params![limit], |row| {
        Ok(TopSong {
            song_id: row.get(0)?, title: row.get(1)?, artist: row.get(2)?,
            play_count: row.get(3)?, total_duration_secs: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
pub fn stats_top_artists(range: String, limit: Option<i64>, db: State<'_, Db>) -> Result<Vec<TopArtist>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let limit = limit.unwrap_or(10);
    let filter = range_filter(&range);

    let mut stmt = conn.prepare(&format!(
        "SELECT s.artist, SUM(s.play_count) as cnt, COALESCE(SUM(ph.duration_secs), 0) as dur, COUNT(DISTINCT s.id) as songs
         FROM play_history ph JOIN songs s ON s.id = ph.song_id WHERE s.artist NOT IN ('Unknown Artist', '') {}
         GROUP BY s.artist ORDER BY dur DESC LIMIT ?", filter
    )).map_err(|e| e.to_string())?;

    let rows: Vec<TopArtist> = stmt.query_map(rusqlite::params![limit], |row| {
        Ok(TopArtist {
            artist: row.get(0)?, play_count: row.get(1)?,
            total_duration_secs: row.get(2)?, song_count: row.get(3)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
pub fn stats_genre_distribution(range: String, db: State<'_, Db>) -> Result<Vec<GenreDuration>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let filter = range_filter(&range);

    let mut stmt = conn.prepare(&format!(
        "SELECT COALESCE(NULLIF(s.genre, ''), 'Unknown') as genre, SUM(ph.duration_secs) as dur
         FROM play_history ph JOIN songs s ON s.id = ph.song_id WHERE 1=1 {}
         GROUP BY genre ORDER BY dur DESC", filter
    )).map_err(|e| e.to_string())?;

    let rows: Vec<GenreDuration> = stmt.query_map([], |row| {
        Ok(GenreDuration { genre: row.get(0)?, duration_secs: row.get(1)? })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
pub fn stats_trend(range: String, db: State<'_, Db>) -> Result<Vec<DayDuration>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let filter = range_filter(&range);

    let mut stmt = conn.prepare(&format!(
        "SELECT DATE(ph.played_at) as day, SUM(ph.duration_secs) as dur
         FROM play_history ph WHERE 1=1 {} GROUP BY day ORDER BY day", filter
    )).map_err(|e| e.to_string())?;

    let rows: Vec<DayDuration> = stmt.query_map([], |row| {
        Ok(DayDuration { date: row.get(0)?, duration_secs: row.get(1)? })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
pub fn stats_heatmap(db: State<'_, Db>) -> Result<Vec<DayDuration>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT DATE(ph.played_at) as day, SUM(ph.duration_secs) as dur
         FROM play_history ph WHERE ph.played_at >= datetime('now', '-365 days')
         GROUP BY day ORDER BY day"
    ).map_err(|e| e.to_string())?;

    let rows: Vec<DayDuration> = stmt.query_map([], |row| {
        Ok(DayDuration { date: row.get(0)?, duration_secs: row.get(1)? })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    Ok(rows)
}
