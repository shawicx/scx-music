use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: String,
    pub duration_secs: f64,
    pub quality: String,
    pub file_path: String,
    pub art_gradient: String,
    pub genre: String,
    pub file_size: i64,
}

impl Song {
    /// 从 songs 表的标准 11 列 SELECT 映射。
    ///
    /// 列顺序（与所有 SELECT songs 查询保持一致）：
    /// 0. id, 1. title, 2. artist, 3. album, 4. duration,
    /// 5. duration_secs, 6. quality, 7. file_path, 8. art_gradient,
    /// 9. genre, 10. file_size
    ///
    /// **修改 SQL 字段顺序时，必须同步更新此函数和文档。**
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
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
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistSong {
    pub playlist_id: String,
    pub song_id: String,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lyric {
    pub song_id: String,
    pub raw_lrc: Option<String>,
    pub source: String,
    /// 歌词时间偏移（秒）。与 lyrics 表 offset_secs 列对应，
    /// 备份/恢复时保留（原 model 遗漏此字段会导致 offset 丢失）。
    #[serde(default)]
    pub offset_secs: f64,
}
