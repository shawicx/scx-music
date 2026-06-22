import { describe, it, expect } from 'vitest'
import { formatCombo } from './keycode'

describe('formatCombo', () => {
  it('空字符串返回空', () => {
    expect(formatCombo('')).toBe('')
  })
  it('mac 平台把 CommandOrControl 映射成 ⌘', () => {
    expect(formatCombo('CommandOrControl+Shift+M', 'mac')).toBe('⌘+⇧+M')
  })
  it('win 平台把 CommandOrControl 映射成 Ctrl', () => {
    expect(formatCombo('CommandOrControl+Shift+M', 'win')).toBe('Ctrl+Shift+M')
  })
  it('剥离 Key 前缀（KeyP → P）', () => {
    expect(formatCombo('CommandOrControl+Shift+KeyP', 'win')).toBe('Ctrl+Shift+P')
  })
  it('媒体键用友好名', () => {
    expect(formatCombo('MediaPlayPause', 'mac')).toBe('Media Play/Pause')
  })
})
