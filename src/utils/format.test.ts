import { describe, it, expect } from 'vitest'
import { formatHoursMinutes, formatTimecode, formatFileSize } from './format'

describe('formatHoursMinutes', () => {
  it('< 60s 向下取整为 0m（无秒位）', () => {
    expect(formatHoursMinutes(42)).toBe('0m')
  })
  it('分钟数向下取整，丢弃秒', () => {
    expect(formatHoursMinutes(125)).toBe('2m') // 2m 5s → '2m'
  })
  it('整小时仍显示分钟部分（0m）', () => {
    expect(formatHoursMinutes(3600)).toBe('1h 0m')
  })
  it('小时 + 分钟组合', () => {
    expect(formatHoursMinutes(3725)).toBe('1h 2m') // 1h 2m 5s
  })
  it('0 秒', () => {
    expect(formatHoursMinutes(0)).toBe('0m')
  })
})

describe('formatTimecode', () => {
  it('m:ss 格式，秒补前导 0', () => {
    expect(formatTimecode(65)).toBe('1:05')
  })
  it('不足 1 分钟显示 0:ss', () => {
    expect(formatTimecode(5)).toBe('0:05')
  })
  it('NaN 返回 0:00', () => {
    expect(formatTimecode(NaN)).toBe('0:00')
  })
  it('Infinity 返回 0:00', () => {
    expect(formatTimecode(Infinity)).toBe('0:00')
  })
})

describe('formatFileSize', () => {
  it('字节单位', () => {
    expect(formatFileSize(500)).toBe('500.0 B')
  })
  it('KB 一位小数', () => {
    expect(formatFileSize(1024)).toBe('1.0 KB')
  })
  it('MB 一位小数', () => {
    expect(formatFileSize(1024 * 1024 * 3.5)).toBe('3.5 MB')
  })
  it('0 字节', () => {
    expect(formatFileSize(0)).toBe('0 B')
  })
})
