import { describe, it, expect } from 'vitest'
import { generateQueue } from './usePlayQueue'
import type { Song, PlaybackMode } from '../types'

function makeSongs(n: number): Song[] {
  return Array.from({ length: n }, (_, i) => ({
    id: `s${i}`,
    title: `Title ${i}`,
    artist: 'Artist',
    album: 'Album',
    duration: '0:00',
    durationSecs: 0,
    quality: 'lossy',
    filePath: `/p/${i}`,
    artGradient: '',
    genre: '',
    fileSize: 0,
  }))
}

describe('generateQueue', () => {
  it('空列表返回空数组（任何模式）', () => {
    const modes: PlaybackMode[] = ['sequential', 'repeat_all', 'repeat_one', 'shuffle']
    for (const m of modes) {
      expect(generateQueue([], 0, m)).toEqual([])
    }
  })
  it('sequential / repeat_all 保留原序', () => {
    const songs = makeSongs(3)
    expect(generateQueue(songs, 0, 'sequential').map((s) => s.id)).toEqual(['s0', 's1', 's2'])
    expect(generateQueue(songs, 0, 'repeat_all').map((s) => s.id)).toEqual(['s0', 's1', 's2'])
  })
  it('repeat_one 只返回当前歌', () => {
    const songs = makeSongs(3)
    const q = generateQueue(songs, 2, 'repeat_one')
    expect(q.map((s) => s.id)).toEqual(['s2'])
  })
  it('shuffle 保留全部元素且当前歌在首位', () => {
    const songs = makeSongs(5)
    const q = generateQueue(songs, 2, 'shuffle')
    expect(q).toHaveLength(5)
    expect(q[0].id).toBe('s2') // 当前歌被挪到首位
    expect([...q.map((s) => s.id)].sort()).toEqual(['s0', 's1', 's2', 's3', 's4']) // 元素不丢
  })
})
