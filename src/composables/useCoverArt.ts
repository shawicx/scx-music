import { invokeCommand } from '../utils/errorHandler'

/**
 * 专辑封面加载（方案 A：IPC raw bytes → Blob URL）。
 *
 * 模块级单例缓存：songId → objectURL，LRU 上限 50 张（淘汰时 revoke）。
 * 失败/无封面静默返回 null（封面缺失是正常态，不是错误，不弹 toast）。
 */

const MAX_CACHE = 50
const urlCache = new Map<string, string>() // Map 迭代序 = 插入序，首位即最旧
const inflight = new Map<string, Promise<string | null>>()

/** 魔数嗅探图片 MIME（后端 raw bytes 不带 mime）。 */
export function sniffImageMime(bytes: Uint8Array): string {
  if (bytes.length >= 4 && bytes[0] === 0x89 && bytes[1] === 0x50 && bytes[2] === 0x4e && bytes[3] === 0x47) {
    return 'image/png'
  }
  if (bytes.length >= 3 && bytes[0] === 0xff && bytes[1] === 0xd8 && bytes[2] === 0xff) {
    return 'image/jpeg'
  }
  if (bytes.length >= 4 && bytes[0] === 0x47 && bytes[1] === 0x49 && bytes[2] === 0x46) {
    return 'image/gif'
  }
  if (
    bytes.length >= 12 &&
    bytes[0] === 0x52 && bytes[1] === 0x49 && bytes[2] === 0x46 && bytes[3] === 0x46 &&
    bytes[8] === 0x57 && bytes[9] === 0x45 && bytes[10] === 0x42 && bytes[11] === 0x50
  ) {
    return 'image/webp'
  }
  return 'image/jpeg'
}

/** 获取封面 blob URL。无封面/失败返回 null。 */
export function getCoverUrl(songId: string): Promise<string | null> {
  const cached = urlCache.get(songId)
  if (cached) {
    // LRU touch：删掉重插，移到最新端
    urlCache.delete(songId)
    urlCache.set(songId, cached)
    return Promise.resolve(cached)
  }

  const existing = inflight.get(songId)
  if (existing) return existing

  const task = (async () => {
    try {
      const buf = await invokeCommand<ArrayBuffer>('get_song_cover', { songId })
      if (!buf || buf.byteLength === 0) return null
      const bytes = new Uint8Array(buf)
      const blob = new Blob([bytes], { type: sniffImageMime(bytes) })
      const url = URL.createObjectURL(blob)
      urlCache.set(songId, url)
      if (urlCache.size > MAX_CACHE) {
        const oldest = urlCache.keys().next().value
        if (oldest !== undefined && oldest !== songId) {
          const oldUrl = urlCache.get(oldest)
          urlCache.delete(oldest)
          if (oldUrl) URL.revokeObjectURL(oldUrl)
        }
      }
      return url
    } catch {
      return null
    } finally {
      inflight.delete(songId)
    }
  })()
  inflight.set(songId, task)
  return task
}

/** 清空前端 URL 缓存并 revoke（清理后端封面缓存后调用，强制重新提取）。 */
export function evictCoverUrlCache(): void {
  for (const url of urlCache.values()) URL.revokeObjectURL(url)
  urlCache.clear()
}

export function useCoverArt() {
  return { getCoverUrl, evictCoverUrlCache }
}
