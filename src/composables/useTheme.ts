import { ref, watch, computed, toRef } from 'vue'
import { useTheme as useVuetifyTheme } from 'vuetify'
import { invoke } from '@tauri-apps/api/core'
import { emit } from '@tauri-apps/api/event'
import { usePreferredDark } from '@vueuse/core'
import type { ThemeColor, ThemeMode } from '../plugins/vuetify'
import { themeMeta } from '../plugins/vuetify'

/** Rust get_cover_accent 返回的封面主色（HSL）。 */
export interface AccentHsl {
  h: number
  s: number
  l: number
}

// ===== 颜色工具 =====

function hexToRgb(hex: string): [number, number, number] {
  const v = hex.replace('#', '')
  return [
    parseInt(v.slice(0, 2), 16),
    parseInt(v.slice(2, 4), 16),
    parseInt(v.slice(4, 6), 16),
  ]
}

function rgbToHsl(r: number, g: number, b: number): AccentHsl {
  r /= 255
  g /= 255
  b /= 255
  const mx = Math.max(r, g, b)
  const mn = Math.min(r, g, b)
  const l = (mx + mn) / 2
  if (mx === mn) return { h: 0, s: 0, l }
  const d = mx - mn
  const s = l > 0.5 ? d / (2 - mx - mn) : d / (mx + mn)
  let h: number
  if (mx === r) h = (g - b) / d + (g < b ? 6 : 0)
  else if (mx === g) h = (b - r) / d + 2
  else h = (r - g) / d + 4
  return { h: h * 60, s, l }
}

function hslToRgb(h: number, s: number, l: number): [number, number, number] {
  const c = (1 - Math.abs(2 * l - 1)) * s
  const x = c * (1 - Math.abs(((h / 60) % 2) - 1))
  const m = l - c / 2
  let r = 0
  let g = 0
  let b = 0
  if (h < 60) [r, g, b] = [c, x, 0]
  else if (h < 120) [r, g, b] = [x, c, 0]
  else if (h < 180) [r, g, b] = [0, c, x]
  else if (h < 240) [r, g, b] = [0, x, c]
  else if (h < 300) [r, g, b] = [x, 0, c]
  else [r, g, b] = [c, 0, x]
  return [Math.round((r + m) * 255), Math.round((g + m) * 255), Math.round((b + m) * 255)]
}

function hslCss(h: number, s: number, l: number): string {
  return `hsl(${h.toFixed(1)} ${(s * 100).toFixed(1)}% ${(l * 100).toFixed(1)}%)`
}

/** 相对亮度，决定强调色上的文字用深还是浅。 */
function luminance(r: number, g: number, b: number): number {
  const f = (c: number) => {
    c /= 255
    return c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4)
  }
  return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b)
}

const clamp = (v: number, a: number, b: number) => Math.min(b, Math.max(a, v))

// Color theme management
export function useColorTheme() {
  const colorName = ref<ThemeColor>('teal')

  async function loadColorFromDb() {
    const value = await invoke<string | null>('get_setting', { key: 'theme_color' })
    if (value) {
      colorName.value = value as ThemeColor
    } else {
      // Migrate old theme setting
      const oldTheme = await invoke<string | null>('get_setting', { key: 'theme' })
      if (oldTheme) {
        colorName.value = oldTheme as ThemeColor
        // Save to new key
        await invoke('set_setting', { key: 'theme_color', value: oldTheme })
      }
    }
  }

  function setColorTheme(name: ThemeColor) {
    colorName.value = name
    invoke('set_setting', { key: 'theme_color', value: name }).catch(console.error)
    // 广播给独立 webview（桌面歌词/迷你播放器），它们有独立 Vuetify 实例需手动同步
    void emit('app:theme-changed', { color: name })
  }

  return { colorName, setColorTheme, loadColorFromDb }
}

// Mode management (light/dark/system)
export function useThemeMode() {
  const mode = ref<ThemeMode>('system')
  const preferredDark = usePreferredDark()

  async function loadModeFromDb() {
    const value = await invoke<string | null>('get_setting', { key: 'theme_mode' })
    if (value) {
      mode.value = value as ThemeMode
    } else {
      // First time user - check if they have old theme setting
      const oldTheme = await invoke<string | null>('get_setting', { key: 'theme' })
      if (oldTheme) {
        // Existing users default to dark mode
        mode.value = 'dark'
        await invoke('set_setting', { key: 'theme_mode', value: 'dark' })
      } else {
        // New users default to system mode
        mode.value = 'system'
      }
    }
  }

  function setMode(newMode: ThemeMode) {
    mode.value = newMode
    invoke('set_setting', { key: 'theme_mode', value: newMode }).catch(console.error)
    // 广播给独立 webview（桌面歌词/迷你播放器），它们有独立 Vuetify 实例需手动同步
    void emit('app:theme-changed', { mode: newMode })
  }

  const isDark = computed(() => {
    if (mode.value === 'system') {
      return preferredDark.value
    }
    return mode.value === 'dark'
  })

  return { mode, setMode, loadModeFromDb, isDark }
}

// ===== 动态取色（模块级单例状态，跨 store/composable 共享） =====

/** 封面提取的原始主色；null = 当前用六色主题回退。 */
const coverAccent = ref<AccentHsl | null>(null)
/** 动态配色开关（设置持久化 dynamic_color）。 */
const dynamicColor = ref(true)
/** 当前生效的强调色（hex，供 ECharts 等非 CSS 场景读取）。 */
const accentHex = ref(themeMeta.teal.color)
/** 强调色广播负载：独立 webview（迷你播放器等）直接消费的预计算值。dark 供其同步 scx-dark/scx-light 标记类。 */
export interface AccentBroadcast {
  hex: string
  rgb: [number, number, number]
  onAccent: string
  npAccent: string
  amb1: string
  amb2: string
  dark: boolean
}
let lastBroadcast: AccentBroadcast | null = null

/** 在独立 webview 中应用广播负载（写 CSS 变量，不依赖本窗口 Vuetify）。 */
export function applyAccentBroadcast(p: AccentBroadcast) {
  const root = document.documentElement.style
  root.setProperty('--accent', p.hex)
  root.setProperty('--accent-rgb', `${p.rgb[0]} ${p.rgb[1]} ${p.rgb[2]}`)
  root.setProperty('--on-accent', p.onAccent)
  root.setProperty('--np-accent', p.npAccent)
  root.setProperty('--amb-1', p.amb1)
  root.setProperty('--amb-2', p.amb2)
}

/** 应用强调色：按明暗钳制后写 CSS 变量并同步 Vuetify primary。 */
function applyAccent(hsl: AccentHsl | null, dark: boolean, fallbackHex: string, vuetifyTheme: ReturnType<typeof useVuetifyTheme>) {
  let rgb: [number, number, number]
  let hue: number
  let sat: number
  if (hsl && hsl.s >= 0.13) {
    // 钳制保证对比度：饱和度 34–72%；明度暗色 55–66%、浅色 34–44%
    hue = ((hsl.h % 360) + 360) % 360
    rgb = hslToRgb(hue, clamp(hsl.s, 0.34, 0.72), dark ? clamp(hsl.l, 0.55, 0.66) : clamp(hsl.l, 0.34, 0.44))
    sat = hsl.s
  } else {
    // 回退六色主题：同样按明暗钳制，保证与封面来源观感一致
    const fb = rgbToHsl(...hexToRgb(fallbackHex))
    hue = fb.h
    sat = fb.s
    rgb = hslToRgb(fb.h, clamp(fb.s, 0.34, 0.72), dark ? clamp(fb.l, 0.55, 0.66) : clamp(fb.l, 0.34, 0.44))
  }
  const hex = `#${rgb.map((v) => v.toString(16).padStart(2, '0')).join('')}`
  accentHex.value = hex
  const onAccent = luminance(...rgb) > 0.42 ? '#17171d' : '#ffffff'
  const npAccent = hslCss(hue, clamp(sat, 0.3, 0.8), dark ? 0.78 : 0.42)
  const amb1 = hslCss(hue, 0.32, dark ? 0.13 : 0.86)
  const amb2 = hslCss((hue + 34) % 360, 0.28, dark ? 0.17 : 0.9)

  const root = document.documentElement.style
  root.setProperty('--accent', hex)
  root.setProperty('--accent-rgb', `${rgb[0]} ${rgb[1]} ${rgb[2]}`)
  root.setProperty('--on-accent', onAccent)
  root.setProperty('--np-accent', npAccent)
  root.setProperty('--amb-1', amb1)
  root.setProperty('--amb-2', amb2)

  // 同步 Vuetify 主题 primary，让 v-switch / v-slider / v-selection-control 自动跟随
  const current = vuetifyTheme.current.value
  if (current) {
    // Vuetify 的 colors/variables 类型标为只读，此处为运行时主题热更新入口
    ;(current.colors as Record<string, string>).primary = hex
    // accent-bg（选中态着色 tint）
    ;(current.variables as Record<string, string>)['accent-bg'] = `rgba(${rgb[0]}, ${rgb[1]}, ${rgb[2]}, 0.14)`
  }

  lastBroadcast = { hex, rgb, onAccent, npAccent, amb1, amb2, dark }
}

/** 按明暗在 <html> 上维护 scx-dark/scx-light 标记类（tokens.css 令牌的挂载点）。 */
function setHtmlThemeClass(dark: boolean) {
  const el = document.documentElement
  el.classList.toggle('scx-dark', dark)
  el.classList.toggle('scx-light', !dark)
}

function broadcastAccent() {
  if (lastBroadcast) {
    void emit('app:accent-changed', lastBroadcast)
  }
}

// Main theme orchestration
export function useTheme() {
  const vuetifyTheme = useVuetifyTheme()
  const { colorName, setColorTheme, loadColorFromDb } = useColorTheme()
  const { mode, setMode, loadModeFromDb, isDark } = useThemeMode()

  // Watch for changes and update Vuetify theme
  watch([colorName, isDark], ([color, dark]) => {
    const themeName = `${color}-${dark ? 'dark' : 'light'}`
    vuetifyTheme.global.name.value = themeName
    // 主题令牌标记类（Vuetify 类名是 v-theme--teal-light 之类，不能用于令牌选择器）
    setHtmlThemeClass(dark)
    // 主题/明暗变化后重新应用强调色（封面来源需按明暗重新钳制；回退来源换色）
    const source = dynamicColor.value ? coverAccent.value : null
    applyAccent(source, dark, themeMeta[color].color, vuetifyTheme)
    broadcastAccent()
  }, { immediate: true })

  async function loadThemeFromDb() {
    await Promise.all([loadColorFromDb(), loadModeFromDb(), loadDynamicColorFromDb()])
  }

  async function loadDynamicColorFromDb() {
    const value = await invoke<string | null>('get_setting', { key: 'dynamic_color' })
    dynamicColor.value = value !== 'false' // 默认开启
    refreshAccent()
  }

  function setDynamicColor(enabled: boolean) {
    dynamicColor.value = enabled
    invoke('set_setting', { key: 'dynamic_color', value: enabled ? 'true' : 'false' }).catch(console.error)
    refreshAccent()
  }

  /** 封面取色结果回填（由歌曲切换监听调用）。低饱和/无封面传 null 走回退。 */
  function setCoverAccent(hsl: AccentHsl | null) {
    coverAccent.value = hsl
    refreshAccent()
  }

  function refreshAccent() {
    const source = dynamicColor.value ? coverAccent.value : null
    applyAccent(source, isDark.value, themeMeta[colorName.value].color, vuetifyTheme)
    broadcastAccent()
  }

  return {
    colorName,
    mode,
    isDark,
    dynamicColor,
    accentHex,
    coverAccent,
    setColorTheme,
    setMode,
    setDynamicColor,
    setCoverAccent,
    refreshAccent,
    loadThemeFromDb,
  }
}

/**
 * 启动封面动态取色监听：跟随当前播放歌曲提取主色。
 * 主窗口 App.vue onMounted 调用（幂等）；独立 webview 用 app:accent-changed 事件同步。
 * player store 延迟加载，避免 settings store -> useTheme 与 player -> settings 的循环依赖。
 */
export async function startCoverAccentWatcher() {
  if (accentWatcherStarted) return
  accentWatcherStarted = true
  const [{ usePlayerStore }, { useSettingsStore }] = await Promise.all([
    import('../stores/player'),
    import('../stores/settings'),
  ])
  const player = usePlayerStore()
  const settings = useSettingsStore()
  const dynamicColor = toRef(() => settings.dynamicColor)
  const setCoverAccent = settings.setCoverAccent
  let requestSeq = 0
  watch(
    () => player.currentSong?.id,
    async (songId) => {
      if (!dynamicColor.value) return
      const seq = ++requestSeq
      if (!songId) {
        setCoverAccent(null)
        return
      }
      try {
        const hsl = await invoke<AccentHsl | null>('get_cover_accent', { songId })
        // 等待期间歌曲已切换：丢弃过期结果
        if (seq !== requestSeq) return
        setCoverAccent(hsl)
      } catch (e) {
        console.error('get_cover_accent failed:', e)
      }
    },
    { immediate: true },
  )
  // 开关变化时对当前歌曲立即重取/重置
  watch(dynamicColor, async (on) => {
    if (!on) {
      setCoverAccent(null)
      return
    }
    const songId = player.currentSong?.id
    if (!songId) return
    try {
      setCoverAccent(await invoke<AccentHsl | null>('get_cover_accent', { songId }))
    } catch {
      setCoverAccent(null)
    }
  })
}

let accentWatcherStarted = false
