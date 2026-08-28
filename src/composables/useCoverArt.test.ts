import { describe, it, expect, vi, beforeEach } from 'vitest'

// mock invokeCommand：默认无封面（空 ArrayBuffer）
const invokeCommandMock = vi.fn<(cmd: string, args?: Record<string, unknown>) => Promise<unknown>>()
vi.mock('../utils/errorHandler', () => ({
  invokeCommand: (cmd: string, args?: Record<string, unknown>) => invokeCommandMock(cmd, args),
}))

import { getCoverUrl, evictCoverUrlCache, sniffImageMime } from './useCoverArt'

// jsdom 无 URL.createObjectURL，stub 之
let blobSeq = 0
beforeEach(() => {
  invokeCommandMock.mockReset()
  invokeCommandMock.mockResolvedValue(new ArrayBuffer(0))
  evictCoverUrlCache()
  blobSeq = 0
  ;(URL as unknown as Record<string, unknown>).createObjectURL = vi.fn(() => `blob:mock-${++blobSeq}`)
  ;(URL as unknown as Record<string, unknown>).revokeObjectURL = vi.fn()
})

const PNG_BYTES = new Uint8Array([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 1, 2])

describe('sniffImageMime', () => {
  it('识别 PNG/JPEG/WebP/GIF 魔数', () => {
    expect(sniffImageMime(new Uint8Array([0x89, 0x50, 0x4e, 0x47, 0x0d]))).toBe('image/png')
    expect(sniffImageMime(new Uint8Array([0xff, 0xd8, 0xff, 0xe0]))).toBe('image/jpeg')
    expect(sniffImageMime(new Uint8Array([0x47, 0x49, 0x46, 0x38]))).toBe('image/gif')
    expect(sniffImageMime(new Uint8Array([0x52, 0x49, 0x46, 0x46, 0, 0, 0, 0, 0x57, 0x45, 0x42, 0x50]))).toBe('image/webp')
  })

  it('未知魔数回退 jpeg', () => {
    expect(sniffImageMime(new Uint8Array([0, 1, 2, 3]))).toBe('image/jpeg')
  })
})

describe('getCoverUrl', () => {
  it('空 body（无封面）返回 null', async () => {
    invokeCommandMock.mockResolvedValue(new ArrayBuffer(0))
    await expect(getCoverUrl('s1')).resolves.toBeNull()
  })

  it('非空 body 返回 blob URL 并缓存', async () => {
    invokeCommandMock.mockResolvedValue(PNG_BYTES.buffer)
    const url1 = await getCoverUrl('s1')
    expect(url1).toBe('blob:mock-1')
    const url2 = await getCoverUrl('s1')
    expect(url2).toBe('blob:mock-1') // 缓存命中，不再 invoke
    expect(invokeCommandMock).toHaveBeenCalledTimes(1)
  })

  it('同 songId 并发调用只发一次 IPC（inflight 去重）', async () => {
    invokeCommandMock.mockResolvedValue(PNG_BYTES.buffer)
    const [a, b] = await Promise.all([getCoverUrl('s1'), getCoverUrl('s1')])
    expect(a).toBe(b)
    expect(invokeCommandMock).toHaveBeenCalledTimes(1)
  })

  it('IPC 失败静默返回 null', async () => {
    invokeCommandMock.mockRejectedValue(new Error('boom'))
    await expect(getCoverUrl('s1')).resolves.toBeNull()
  })

  it('evictCoverUrlCache 清空缓存并 revoke', async () => {
    invokeCommandMock.mockResolvedValue(PNG_BYTES.buffer)
    await getCoverUrl('s1')
    evictCoverUrlCache()
    await getCoverUrl('s1')
    expect(invokeCommandMock).toHaveBeenCalledTimes(2)
    expect(URL.revokeObjectURL).toHaveBeenCalledWith('blob:mock-1')
  })
})
