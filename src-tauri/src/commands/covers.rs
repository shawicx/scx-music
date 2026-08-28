//! 专辑封面按需提取与文件缓存。
//!
//! 管线：查 songs 表拿 file_path → 缓存目录查 `{song_id}.{ext}` →
//! 未命中用 Lofty 提取内嵌 picture → 写缓存；无内嵌封面写 `{song_id}.none`
//! 负缓存标记（避免每次切歌重复解析音频文件）。
//!
//! 设计：核心逻辑为接收 `&Path`/`&Connection` 的纯函数，命令层
//! 负责 State/AppHandle 参数；本文件不持锁，`?` 直传 AppError。

use crate::db::Db;
use crate::error::AppResult;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::path::{Path, PathBuf};

/// 封面缓存统计。
#[derive(Serialize, Default, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CoverCacheStats {
    pub total: i64,
    pub size_bytes: i64,
}

/// 查歌曲的 file_path。歌曲不存在返回 Ok(None)。
pub(crate) fn song_file_path(conn: &Connection, song_id: &str) -> AppResult<Option<String>> {
    let mut stmt = conn.prepare("SELECT file_path FROM songs WHERE id = ?1")?;
    let mut rows = stmt.query(params![song_id])?;
    match rows.next()? {
        Some(row) => Ok(Some(row.get(0)?)),
        None => Ok(None),
    }
}

/// 用 Lofty 提取内嵌封面。解析失败/无 picture 一律返回 None（负缓存语义：
/// 提取失败与确认无封面不作区分，均不再重试）。
fn extract_embedded_cover(file_path: &str) -> Option<(Vec<u8>, String)> {
    use lofty::config::ParseOptions;
    use lofty::file::TaggedFileExt;
    use lofty::probe::Probe;

    let file = std::fs::File::open(file_path).ok()?;
    // read_properties(false)：只读元数据区，跳过音频帧扫描，大文件更快
    let tagged = Probe::new(file)
        .options(ParseOptions::new().read_properties(false))
        .guess_file_type() // read() 不会自动猜格式，必须显式调用
        .ok()?
        .read()
        .ok()?;
    let tag = tagged.primary_tag()?;
    let pic = tag.pictures().first()?;
    let mime = pic
        .mime_type()
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| "image/jpeg".to_string());
    Some((pic.data().to_vec(), mime))
}

/// mime → 缓存文件扩展名。
fn ext_for_mime(mime: &str) -> &'static str {
    match mime {
        "image/png" => "png",
        "image/webp" => "webp",
        "image/tiff" => "tif",
        "image/bmp" => "bmp",
        "image/gif" => "gif",
        _ => "jpg",
    }
}

/// 缓存命中：扫描目录找 `{song_id}.{ext}`（ext 未知，需前缀匹配）。
fn cached_cover(covers_dir: &Path, song_id: &str) -> Option<Vec<u8>> {
    let prefix = format!("{song_id}.");
    for entry in std::fs::read_dir(covers_dir).ok()? {
        let entry = entry.ok()?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with(&prefix) && !name.ends_with(".none") {
            return std::fs::read(entry.path()).ok();
        }
    }
    None
}

fn negative_cache_exists(covers_dir: &Path, song_id: &str) -> bool {
    covers_dir.join(format!("{song_id}.none")).exists()
}

/// 主入口：拿封面字节（含缓存/提取/负缓存写回）。写缓存失败静默降级为不缓存。
pub(crate) fn cover_bytes_for(covers_dir: &Path, file_path: &str, song_id: &str) -> Option<Vec<u8>> {
    if negative_cache_exists(covers_dir, song_id) {
        return None;
    }
    if let Some(bytes) = cached_cover(covers_dir, song_id) {
        return Some(bytes);
    }
    match extract_embedded_cover(file_path) {
        Some((bytes, mime)) => {
            let path = covers_dir.join(format!("{song_id}.{}", ext_for_mime(&mime)));
            let _ = std::fs::write(path, &bytes);
            Some(bytes)
        },
        None => {
            let _ = std::fs::write(covers_dir.join(format!("{song_id}.none")), b"");
            None
        },
    }
}

/// 统计：目录内全部文件（含 .none 标记）的数量与总字节。
pub(crate) fn cover_cache_stats_inner(covers_dir: &Path) -> CoverCacheStats {
    let mut stats = CoverCacheStats::default();
    let Ok(entries) = std::fs::read_dir(covers_dir) else {
        return stats;
    };
    for entry in entries.flatten() {
        let Ok(meta) = entry.metadata() else { continue };
        if !meta.is_file() {
            continue;
        }
        stats.total += 1;
        stats.size_bytes += meta.len() as i64;
    }
    stats
}

/// 清理：删除目录内全部文件，返回删除数。目录不存在视为空。
pub(crate) fn clear_cover_cache_inner(covers_dir: &Path) -> AppResult<crate::commands::cache::ClearedResult> {
    let mut cleared: i64 = 0;
    for entry in std::fs::read_dir(covers_dir)? {
        let entry = entry?;
        if entry.metadata()?.is_file() {
            std::fs::remove_file(entry.path())?;
            cleared += 1;
        }
    }
    Ok(crate::commands::cache::ClearedResult { cleared })
}

// ===== 命令层（薄包装：State/AppHandle → 纯函数参数） =====

/// 缓存目录路径（不存在则创建）。
fn covers_dir(app: &tauri::AppHandle) -> AppResult<PathBuf> {
    use tauri::Manager;
    let dir = app.path().app_cache_dir()?.join("covers");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// 获取歌曲封面（raw bytes）。空 body = 无封面（歌曲不存在/无内嵌封面/负缓存）。
///
/// 用 `tauri::ipc::Response` 走 raw 通道（Vec<u8> 直传，前端收 ArrayBuffer），
/// 避免封面字节被 serde JSON 数列化（~4 倍膨胀）。
#[tauri::command]
pub async fn get_song_cover(
    db: tauri::State<'_, Db>,
    app: tauri::AppHandle,
    song_id: String,
) -> AppResult<tauri::ipc::Response> {
    let file_path = {
        let conn = crate::audio::lock_or_recover(&db.0);
        match song_file_path(&conn, &song_id)? {
            Some(p) => p,
            None => return Ok(tauri::ipc::Response::new(Vec::new())),
        }
    };
    let dir = covers_dir(&app)?;
    let bytes = cover_bytes_for(&dir, &file_path, &song_id).unwrap_or_default();
    Ok(tauri::ipc::Response::new(bytes))
}

#[tauri::command]
pub fn get_cover_cache_stats(app: tauri::AppHandle) -> AppResult<CoverCacheStats> {
    let dir = covers_dir(&app)?;
    Ok(cover_cache_stats_inner(&dir))
}

#[tauri::command]
pub fn clear_cover_cache(app: tauri::AppHandle) -> AppResult<crate::commands::cache::ClearedResult> {
    let dir = covers_dir(&app)?;
    clear_cover_cache_inner(&dir)
}

#[cfg(test)]
mod tests {
    use super::*;

// ===== 测试 =====

/// 临时缓存目录（测试隔离：进程 id + 用例名）。
fn temp_covers_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("scx-covers-test-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// 构造最小合法 FLAC：fLaC 魔数 + STREAMINFO + PICTURE 块（lofty 可解析）。
fn build_flac_with_picture(png: &[u8]) -> Vec<u8> {
    fn push_block(buf: &mut Vec<u8>, last: bool, ty: u8, body: &[u8]) {
        buf.push(((last as u8) << 7) | ty);
        buf.extend_from_slice(&(body.len() as u32).to_be_bytes()[1..4]);
        buf.extend_from_slice(body);
    }

    let mut buf = b"fLaC".to_vec();

    // STREAMINFO（34 字节）
    let mut si = Vec::with_capacity(34);
    si.extend_from_slice(&4096u16.to_be_bytes()); // min blocksize（规范要求 ≥16）
    si.extend_from_slice(&4096u16.to_be_bytes()); // max blocksize
    si.extend_from_slice(&[0u8; 3]); // min framesize（未知）
    si.extend_from_slice(&[0u8; 3]); // max framesize（未知）
    let packed: u64 = (44100u64 << 44) | (1u64 << 41) | (15u64 << 36); // 44.1kHz 2ch 16bit
    si.extend_from_slice(&packed.to_be_bytes());
    si.extend_from_slice(&[0u8; 16]); // MD5
    push_block(&mut buf, false, 0, &si);

    // PICTURE（front cover）
    let mime = b"image/png";
    let mut pic = Vec::new();
    pic.extend_from_slice(&3u32.to_be_bytes());
    pic.extend_from_slice(&(mime.len() as u32).to_be_bytes());
    pic.extend_from_slice(mime);
    pic.extend_from_slice(&0u32.to_be_bytes()); // 描述长度
    pic.extend_from_slice(&1u32.to_be_bytes()); // 宽
    pic.extend_from_slice(&1u32.to_be_bytes()); // 高
    pic.extend_from_slice(&24u32.to_be_bytes()); // 位深
    pic.extend_from_slice(&0u32.to_be_bytes()); // 颜色数
    pic.extend_from_slice(&(png.len() as u32).to_be_bytes());
    pic.extend_from_slice(png);
    push_block(&mut buf, true, 6, &pic);

    buf
}

    const PNG_MAGIC: &[u8] = &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 1, 2, 3];

    #[test]
    fn extracts_embedded_cover_and_writes_cache() {
        let dir = temp_covers_dir("extract");
        let audio = dir.join("a.flac");
        std::fs::write(&audio, build_flac_with_picture(PNG_MAGIC)).unwrap();

        let bytes = cover_bytes_for(&dir, audio.to_str().unwrap(), "song-1");
        assert_eq!(bytes.as_deref(), Some(PNG_MAGIC));
        assert!(dir.join("song-1.png").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn second_call_hits_cache_without_reparsing() {
        let dir = temp_covers_dir("hit");
        let audio = dir.join("a.flac");
        std::fs::write(&audio, build_flac_with_picture(PNG_MAGIC)).unwrap();

        cover_bytes_for(&dir, audio.to_str().unwrap(), "song-1").unwrap();
        // 篡改音频内容：缓存命中应返回旧字节而非重新提取
        std::fs::write(&audio, b"garbage").unwrap();
        let bytes = cover_bytes_for(&dir, audio.to_str().unwrap(), "song-1");
        assert_eq!(bytes.as_deref(), Some(PNG_MAGIC));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn no_cover_writes_negative_marker() {
        let dir = temp_covers_dir("none");
        let audio = dir.join("a.flac");
        // 只有 STREAMINFO 的 FLAC（无 PICTURE 块）
        let mut buf = b"fLaC".to_vec();
        let mut si = vec![0u8; 34];
        si[0..2].copy_from_slice(&4096u16.to_be_bytes());
        si[2..4].copy_from_slice(&4096u16.to_be_bytes());
        buf.push(0x80); // last=true, type=0
        buf.extend_from_slice(&34u32.to_be_bytes()[1..4]);
        buf.extend_from_slice(&si);
        std::fs::write(&audio, &buf).unwrap();

        assert_eq!(cover_bytes_for(&dir, audio.to_str().unwrap(), "song-1"), None);
        assert!(dir.join("song-1.none").exists());
        // 负缓存后即使文件换成有封面的也不再解析
        std::fs::write(&audio, build_flac_with_picture(PNG_MAGIC)).unwrap();
        assert_eq!(cover_bytes_for(&dir, audio.to_str().unwrap(), "song-1"), None);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_audio_file_is_negative_cached() {
        let dir = temp_covers_dir("missing");
        assert_eq!(cover_bytes_for(&dir, "/nonexistent/x.flac", "song-1"), None);
        assert!(dir.join("song-1.none").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn stats_counts_files_and_bytes() {
        let dir = temp_covers_dir("stats");
        std::fs::write(dir.join("a.png"), [1u8; 10]).unwrap();
        std::fs::write(dir.join("b.none"), []).unwrap();
        let stats = cover_cache_stats_inner(&dir);
        assert_eq!(stats.total, 2);
        assert_eq!(stats.size_bytes, 10);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn clear_removes_all_files() {
        let dir = temp_covers_dir("clear");
        std::fs::write(dir.join("a.png"), [1u8; 4]).unwrap();
        std::fs::write(dir.join("b.none"), []).unwrap();
        let result = clear_cover_cache_inner(&dir).unwrap();
        assert_eq!(result.cleared, 2);
        assert_eq!(cover_cache_stats_inner(&dir).total, 0);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
