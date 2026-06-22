import { describe, it, expect } from 'vitest'
import { parseLrc } from './lyrics'

describe('parseLrc', () => {
  it('解析单行带 2 位毫秒', () => {
    const result = parseLrc('[01:02.50]hello')
    expect(result).toEqual([{ time: 62.5, text: 'hello' }])
  })
  it('解析单行带 3 位毫秒', () => {
    const result = parseLrc('[00:05.250]world')
    expect(result).toEqual([{ time: 5.25, text: 'world' }])
  })
  it('跳过无时间戳的行', () => {
    const result = parseLrc('no tag here\n[00:01.00]ok')
    expect(result).toEqual([{ time: 1, text: 'ok' }])
  })
  it('按时间升序排序', () => {
    const result = parseLrc('[00:10.00]b\n[00:05.00]a')
    expect(result.map((l) => l.text)).toEqual(['a', 'b'])
  })
})
