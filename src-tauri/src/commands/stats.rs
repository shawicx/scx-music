use crate::db::Db;
use serde::Serialize;

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
