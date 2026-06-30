//! 缓存与冗余数据清理命令。
//!
//! 清理对象：
//! - 歌词缓存（lyrics 表，含 source='none' 负缓存）
//! - 孤儿歌词（song_id 已不在 songs 表的残留行）
//! - 播放历史（play_history 表，按时间段）
//!
//! 设计：核心 SQL 提取为接收 `&Connection` 的纯函数（`_inner` 后缀），
//! 命令做薄包装调 `lock_or_recover`，便于用内存 SQLite 单元测试。

use crate::db::Db;
use crate::error::{AppError, AppResult};
use rusqlite::{params, Connection};
use serde::Serialize;

/// 歌词缓存统计。
#[derive(Serialize, Default, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LyricsCacheStats {
    pub total: i64,
    pub size_bytes: i64,
    pub orphan_count: i64,
    pub by_source: BySource,
}

#[derive(Serialize, Default, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BySource {
    pub embedded: i64,
    pub lrclib: i64,
    pub none: i64,
}

/// 播放历史统计。
#[derive(Serialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlayHistoryStats {
    pub total: i64,
    pub oldest_at: Option<String>,
    pub size_bytes: i64,
}

/// 清理操作的返回值。
#[derive(Serialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClearedResult {
    pub cleared: i64,
}

/// 播放历史清理返回值（带清理范围标识）。
#[derive(Serialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HistoryClearedResult {
    pub cleared: i64,
    pub scope: String,
}

#[tauri::command]
pub fn get_lyrics_cache_stats(db: tauri::State<'_, Db>) -> AppResult<LyricsCacheStats> {
    let conn = crate::audio::lock_or_recover(&db.0);
    get_lyrics_cache_stats_inner(&conn)
}

#[tauri::command]
pub fn get_play_history_stats(db: tauri::State<'_, Db>) -> AppResult<PlayHistoryStats> {
    let conn = crate::audio::lock_or_recover(&db.0);
    get_play_history_stats_inner(&conn)
}

#[tauri::command]
pub fn clear_lyrics_cache(db: tauri::State<'_, Db>) -> AppResult<ClearedResult> {
    let conn = crate::audio::lock_or_recover(&db.0);
    clear_lyrics_cache_inner(&conn)
}

#[tauri::command]
pub fn clear_orphan_lyrics(db: tauri::State<'_, Db>) -> AppResult<ClearedResult> {
    let conn = crate::audio::lock_or_recover(&db.0);
    clear_orphan_lyrics_inner(&conn)
}

#[tauri::command]
pub fn clear_play_history(
    db: tauri::State<'_, Db>,
    before_days: Option<i64>,
) -> AppResult<HistoryClearedResult> {
    let conn = crate::audio::lock_or_recover(&db.0);
    clear_play_history_inner(&conn, before_days)
}

// ===== 纯函数核心（接收 &Connection，便于测试） =====
// 在后续 Task 中实现

fn get_lyrics_cache_stats_inner(conn: &Connection) -> AppResult<LyricsCacheStats> {
    let total: i64 = conn.query_row("SELECT COUNT(*) FROM lyrics", [], |row| row.get(0))?;
    let size_bytes: i64 = conn.query_row(
        "SELECT COALESCE(SUM(length(raw_lrc)), 0) FROM lyrics",
        [],
        |row| row.get(0),
    )?;
    let orphan_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM lyrics WHERE song_id NOT IN (SELECT id FROM songs)",
        [],
        |row| row.get(0),
    )?;
    let by_source = query_by_source(conn)?;
    Ok(LyricsCacheStats {
        total,
        size_bytes,
        orphan_count,
        by_source,
    })
}

/// 按 source 分组计数歌词。新增 source 值不在此三类则计入 0（当前业务仅这三类）。
fn query_by_source(conn: &Connection) -> AppResult<BySource> {
    let embedded = count_source(conn, "embedded")?;
    let lrclib = count_source(conn, "lrclib")?;
    let none = count_source(conn, "none")?;
    Ok(BySource {
        embedded,
        lrclib,
        none,
    })
}

fn count_source(conn: &Connection, source: &str) -> AppResult<i64> {
    Ok(conn.query_row(
        "SELECT COUNT(*) FROM lyrics WHERE source = ?1",
        params![source],
        |row| row.get(0),
    )?)
}

/// play_history 行平均字节数估算（id 8 + song_id 36 + played_at 19 + duration 8 ≈ 71，取保守 80）。
/// 仅为 UI 展示"约 X KB"用，非精确磁盘占用。
const ESTIMATED_HISTORY_ROW_BYTES: i64 = 80;

fn get_play_history_stats_inner(conn: &Connection) -> AppResult<PlayHistoryStats> {
    let total: i64 = conn.query_row("SELECT COUNT(*) FROM play_history", [], |row| row.get(0))?;
    let oldest_at: Option<String> = conn
        .query_row("SELECT MIN(played_at) FROM play_history", [], |row| row.get(0))
        .ok()
        .flatten();
    let size_bytes = total * ESTIMATED_HISTORY_ROW_BYTES;
    Ok(PlayHistoryStats {
        total,
        oldest_at,
        size_bytes,
    })
}

fn clear_lyrics_cache_inner(conn: &Connection) -> AppResult<ClearedResult> {
    let cleared = conn.execute("DELETE FROM lyrics", [])?;
    Ok(ClearedResult {
        cleared: cleared as i64,
    })
}

fn clear_orphan_lyrics_inner(conn: &Connection) -> AppResult<ClearedResult> {
    let cleared = conn.execute(
        "DELETE FROM lyrics WHERE song_id NOT IN (SELECT id FROM songs)",
        [],
    )?;
    Ok(ClearedResult {
        cleared: cleared as i64,
    })
}

/// 按时间段清理播放历史。
///
/// - `before_days = None` → 清空全部，scope = "all"
/// - `before_days = Some(n)`（n > 0）→ 删除 `played_at < datetime('now', '-n days')`，scope = "before_{n}d"
/// - `before_days = Some(0)` 或负数 → `InvalidArgument` 错误
///
/// 用 SQLite 内置 `datetime('now', ?)` 计算阈值，避免前后端时区不一致。
fn clear_play_history_inner(
    conn: &Connection,
    before_days: Option<i64>,
) -> AppResult<HistoryClearedResult> {
    let (cleared, scope) = match before_days {
        None => {
            let c = conn.execute("DELETE FROM play_history", [])?;
            (c, "all".to_string())
        }
        Some(n) if n > 0 => {
            // SQLite datetime 修饰符：负天数表示往前推（-30 days = 30 天前）
            let modifier = format!("-{} days", n);
            let c = conn.execute(
                "DELETE FROM play_history WHERE played_at < datetime('now', ?1)",
                params![modifier],
            )?;
            (c, format!("before_{}d", n))
        }
        _ => {
            return Err(AppError::InvalidArgument(format!(
                "before_days 必须为正数或 null，收到: {:?}",
                before_days
            )))
        }
    };
    Ok(HistoryClearedResult {
        cleared: cleared as i64,
        scope,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 建内存 SQLite 并跑迁移，返回带表结构的连接。
    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run_migrations(&conn).unwrap();
        conn
    }

    /// 插入一条 song 行（最小字段，规避 NOT NULL 约束）。
    fn insert_song(conn: &Connection, id: &str) {
        conn.execute(
            "INSERT INTO songs (id, title, duration, duration_secs, file_path) VALUES (?1, ?1, '0:00', 0.0, ?1)",
            params![id],
        )
        .unwrap();
    }

    /// 插入一条 lyric 行。
    fn insert_lyric(conn: &Connection, song_id: &str, raw_lrc: Option<&str>, source: &str) {
        conn.execute(
            "INSERT INTO lyrics (song_id, raw_lrc, source, offset_secs) VALUES (?1, ?2, ?3, 0.0)",
            params![song_id, raw_lrc, source],
        )
        .unwrap();
    }

    #[test]
    fn lyrics_stats_counts_all_sources_and_orphans() {
        let conn = test_db();
        insert_song(&conn, "s1");
        insert_song(&conn, "s2");
        // s1 embedded, s2 lrclib, 孤儿 s3 (无对应 song), none 负缓存 s4 (无对应 song)
        insert_lyric(&conn, "s1", Some("[00:00.00]a"), "embedded");
        insert_lyric(&conn, "s2", Some("[00:01.00]b"), "lrclib");
        insert_lyric(&conn, "s3", Some("[00:02.00]cc"), "lrclib");
        insert_lyric(&conn, "s4", None, "none");

        let stats = get_lyrics_cache_stats_inner(&conn).unwrap();
        assert_eq!(stats.total, 4);
        assert_eq!(stats.by_source.embedded, 1);
        assert_eq!(stats.by_source.lrclib, 2);
        assert_eq!(stats.by_source.none, 1);
        // 孤儿 = s3, s4（无对应 song）
        assert_eq!(stats.orphan_count, 2);
        // size_bytes = 各 raw_lrc 字节长之和（None 计 0）
        // "[00:00.00]a"=11, "[00:01.00]b"=11, "[00:02.00]cc"=12, None=0
        assert_eq!(stats.size_bytes, 11 + 11 + 12);
    }

    #[test]
    fn lyrics_stats_empty_table_returns_zeros() {
        let conn = test_db();
        let stats = get_lyrics_cache_stats_inner(&conn).unwrap();
        assert_eq!(stats.total, 0);
        assert_eq!(stats.size_bytes, 0);
        assert_eq!(stats.orphan_count, 0);
        assert_eq!(stats.by_source, BySource::default());
    }

    #[test]
    fn play_history_stats_counts_and_oldest() {
        let conn = test_db();
        insert_song(&conn, "s1");
        conn.execute(
            "INSERT INTO play_history (song_id, played_at, duration_secs) VALUES ('s1', '2025-01-03 10:00:00', 60.0)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO play_history (song_id, played_at, duration_secs) VALUES ('s1', '2026-06-01 10:00:00', 120.0)",
            [],
        )
        .unwrap();

        let stats = get_play_history_stats_inner(&conn).unwrap();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.oldest_at.as_deref(), Some("2025-01-03 10:00:00"));
        // size_bytes = total * ESTIMATED_ROW_BYTES（常量，见实现）
        assert!(stats.size_bytes > 0);
    }

    #[test]
    fn play_history_stats_empty_returns_none_oldest() {
        let conn = test_db();
        let stats = get_play_history_stats_inner(&conn).unwrap();
        assert_eq!(stats.total, 0);
        assert_eq!(stats.oldest_at, None);
        assert_eq!(stats.size_bytes, 0);
    }

    #[test]
    fn clear_lyrics_cache_removes_all_including_negative() {
        let conn = test_db();
        // 注意：lyrics.song_id 为 PRIMARY KEY（见 migrations.rs），一首歌一行歌词，
        // 故需用不同 song 验证“清空含 source='none' 负缓存”。
        insert_song(&conn, "s1");
        insert_song(&conn, "s2");
        insert_lyric(&conn, "s1", Some("[00:00.00]a"), "embedded");
        insert_lyric(&conn, "s2", None, "none");

        let result = clear_lyrics_cache_inner(&conn).unwrap();
        assert_eq!(result.cleared, 2);

        // 全部清空（含 source='none' 负缓存）
        let remaining: i64 =
            conn.query_row("SELECT COUNT(*) FROM lyrics", [], |row| row.get(0))
                .unwrap();
        assert_eq!(remaining, 0);
    }

    #[test]
    fn clear_orphan_lyrics_keeps_valid_removes_orphans() {
        let conn = test_db();
        insert_song(&conn, "s1");
        insert_song(&conn, "s2");
        insert_lyric(&conn, "s1", Some("[00:00.00]valid"), "embedded");
        insert_lyric(&conn, "s2", Some("[00:01.00]valid2"), "lrclib");
        // 孤儿：s3 / s4 无对应 song
        insert_lyric(&conn, "s3", Some("[00:02.00]orphan"), "lrclib");
        insert_lyric(&conn, "s4", None, "none");

        let result = clear_orphan_lyrics_inner(&conn).unwrap();
        assert_eq!(result.cleared, 2);

        // 有效歌词保留
        let remaining: i64 =
            conn.query_row("SELECT COUNT(*) FROM lyrics", [], |row| row.get(0))
                .unwrap();
        assert_eq!(remaining, 2);

        // 剩下的都是 s1 / s2
        let orphan_remaining: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM lyrics WHERE song_id NOT IN (SELECT id FROM songs)",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(orphan_remaining, 0);
    }

    #[test]
    fn clear_play_history_all_clears_everything() {
        let conn = test_db();
        insert_song(&conn, "s1");
        conn.execute(
            "INSERT INTO play_history (song_id, played_at, duration_secs) VALUES ('s1', '2020-01-01 00:00:00', 10.0)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO play_history (song_id, played_at, duration_secs) VALUES ('s1', '2026-06-29 00:00:00', 20.0)",
            [],
        )
        .unwrap();

        let result = clear_play_history_inner(&conn, None).unwrap();
        assert_eq!(result.cleared, 2);
        assert_eq!(result.scope, "all");

        let remaining: i64 =
            conn.query_row("SELECT COUNT(*) FROM play_history", [], |row| row.get(0))
                .unwrap();
        assert_eq!(remaining, 0);
    }

    #[test]
    fn clear_play_history_by_days_keeps_recent() {
        let conn = test_db();
        insert_song(&conn, "s1");
        // 旧记录（远超 30 天）
        conn.execute(
            "INSERT INTO play_history (song_id, played_at, duration_secs) VALUES ('s1', '2020-01-01 00:00:00', 10.0)",
            [],
        )
        .unwrap();
        // 新记录（今天）
        conn.execute(
            "INSERT INTO play_history (song_id, played_at, duration_secs) VALUES ('s1', datetime('now'), 20.0)",
            [],
        )
        .unwrap();

        let result = clear_play_history_inner(&conn, Some(30)).unwrap();
        assert_eq!(result.cleared, 1, "应只删 30 天前的旧记录");
        assert_eq!(result.scope, "before_30d");

        let remaining: i64 =
            conn.query_row("SELECT COUNT(*) FROM play_history", [], |row| row.get(0))
                .unwrap();
        assert_eq!(remaining, 1, "应保留今天的新记录");
    }

    #[test]
    fn clear_play_history_rejects_zero_and_negative() {
        let conn = test_db();
        assert!(clear_play_history_inner(&conn, Some(0)).is_err());
        assert!(clear_play_history_inner(&conn, Some(-5)).is_err());
    }
}
