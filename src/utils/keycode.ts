/**
 * Tauri global-hotkey 的媒体/音频键代码 → 友好展示
 * 故意不与 action label 重合，让用户能区分「动作描述」和「物理键位」
 */
const KEY_DISPLAY: Record<string, string> = {
  MediaPlayPause: 'Media Play/Pause',
  MediaTrackNext: 'Media Next',
  MediaTrackPrevious: 'Media Previous',
  MediaStop: 'Media Stop',
  AudioVolumeUp: 'Volume Up',
  AudioVolumeDown: 'Volume Down',
  AudioMute: 'Mute',
}

/**
 * 把 Tauri globalShortcut 的组合字符串转成平台友好的展示形式
 * 输入示例：'CommandOrControl+Shift+M'
 *   macOS 输出：'⌘+Shift+M'
 *   Windows 输出：'Ctrl+Shift+M'
 * 输入示例：'MediaPlayPause'
 *   输出：'Media Play/Pause'
 * 输入示例：'CommandOrControl+Shift+KeyM'
 *   输出：'⌘+Shift+M'（Key 前缀被剥离）
 */
export function formatCombo(
  combo: string,
  platform: 'mac' | 'win' = detectPlatform(),
): string {
  if (!combo) return ''
  return combo
    .split('+')
    .map(part => {
      const trimmed = part.trim()
      if (trimmed === 'CommandOrControl' || trimmed === 'Ctrl' || trimmed === 'Cmd') {
        return platform === 'mac' ? '⌘' : 'Ctrl'
      }
      if (trimmed === 'Alt') return platform === 'mac' ? '⌥' : 'Alt'
      if (trimmed === 'Shift') return platform === 'mac' ? '⇧' : 'Shift'
      if (trimmed === 'Super' || trimmed === 'Meta') return platform === 'mac' ? '⌃' : 'Win'
      if (KEY_DISPLAY[trimmed]) return KEY_DISPLAY[trimmed]
      if (/^Key[A-Z]$/.test(trimmed)) return trimmed.slice(3)
      if (/^Digit\d$/.test(trimmed)) return trimmed.slice(5)
      return trimmed
    })
    .join('+')
}

/** 简单的平台检测 — 在 Tauri web 上下文外默认 mac */
export function detectPlatform(): 'mac' | 'win' {
  if (typeof navigator === 'undefined') return 'mac'
  const ua = navigator.userAgent.toLowerCase()
  if (ua.includes('win')) return 'win'
  return 'mac'
}

/** 反向：用户在捕获时按下组合键，构造 Tauri 可识别的字符串 */
export function buildComboFromEvent(e: KeyboardEvent): string {
  const parts: string[] = []
  if (e.metaKey || e.ctrlKey) parts.push('CommandOrControl')
  if (e.shiftKey) parts.push('Shift')
  if (e.altKey) parts.push('Alt')

  // 修饰键本身不作为主键
  const key = e.key
  if (['Meta', 'Control', 'Shift', 'Alt'].includes(key)) {
    return ''  // 还没按完
  }

  // 媒体键直接使用 key 名（KeyboardEvent.key 已经是 'MediaPlayPause' 等）
  if (key.startsWith('Media') || key.startsWith('Audio')) {
    parts.push(key)
    return parts.join('+')
  }

  // 普通键：单字母大写，数字保留
  if (key.length === 1) {
    parts.push('Key' + key.toUpperCase())
  } else {
    // 例如 'Enter' / 'Escape' / 'ArrowUp' — Tauri 用 KeyEnter / ArrowUp 等
    // 完整映射见 https://docs.rs/tauri-plugin-global-shortcut/latest/tauri_plugin_global_shortcut/enum.Code.html
    parts.push(key)
  }
  return parts.join('+')
}
