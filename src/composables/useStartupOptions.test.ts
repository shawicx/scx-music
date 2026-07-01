import { describe, it, expect } from 'vitest'
import { rebuildSourceList, parsePosition, shouldSavePosition } from './useStartupOptions'
import type { Song } from '../types'

function makeSong(id: string): Song {
  return {
    id,
    title: id,
    artist: 'A',
    album: 'Al',
    duration: '0:00',
    durationSecs: 100,
    quality: 'MP3',
    filePath: `/p/${id}`,
    artGradient: 'gradient-0',
    genre: '',
    fileSize: 0,
  }
}

describe('rebuildSourceList', () => {
  it('无 playlistId 时返回全部曲库', () => {
    const songs = [makeSong('1'), makeSong('2')]
    expect(rebuildSourceList(songs, {}, null)).toEqual(songs)
  })

  it('有 playlistId 时按映射过滤', () => {
    const songs = [makeSong('1'), makeSong('2'), makeSong('3')]
    const map = { p1: ['1', '3'] }
    const result = rebuildSourceList(songs, map, 'p1')
    expect(result.map((s) => s.id)).toEqual(['1', '3'])
  })

  it('playlistId 指向已删除歌单时返回空数组', () => {
    const songs = [makeSong('1')]
    expect(rebuildSourceList(songs, {}, 'gone')).toEqual([])
  })

  it('歌单映射为空数组时返回空', () => {
    const songs = [makeSong('1')]
    expect(rebuildSourceList(songs, { p1: [] }, 'p1')).toEqual([])
  })

  it('歌单包含不存在的 songId 时安全跳过', () => {
    const songs = [makeSong('1')]
    const map = { p1: ['1', 'ghost'] }
    expect(rebuildSourceList(songs, map, 'p1').map((s) => s.id)).toEqual(['1'])
  })
})

describe('parsePosition', () => {
  it('正常数字字符串解析为秒', () => {
    expect(parsePosition('42.5')).toBe(42.5)
  })

  it('空字符串返回 0', () => {
    expect(parsePosition('')).toBe(0)
  })

  it('非法字符串返回 0', () => {
    expect(parsePosition('abc')).toBe(0)
  })

  it('负数返回 0', () => {
    expect(parsePosition('-5')).toBe(0)
  })

  it('超过时长的位置被钳制为时长', () => {
    expect(parsePosition('200', 100)).toBe(100)
  })

  it('不传 duration 时不做上限钳制', () => {
    expect(parsePosition('999')).toBe(999)
  })
})

describe('shouldSavePosition', () => {
  it('距上次保存超过阈值返回 true', () => {
    expect(shouldSavePosition(5001, 0, 5000)).toBe(true)
  })

  it('距上次保存不足阈值返回 false', () => {
    expect(shouldSavePosition(3000, 0, 5000)).toBe(false)
  })
})
