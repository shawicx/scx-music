import { ref, reactive, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  getCurrentWindow,
  LogicalPosition,
  LogicalSize,
  PhysicalPosition,
  currentMonitor,
  type Window,
} from '@tauri-apps/api/window'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { listen, emit, type UnlistenFn } from '@tauri-apps/api/event'
import { useLyrics, type LrcLine } from './useLyrics'
import type { Song } from '../types'

export type GlowStrength = 'off' | 'weak' | 'medium' | 'strong'

export interface DesktopLyricsConfig {
  bgOpacity: number
  fontSize: number
  colorCurrent: string
  colorNext: string
  glowStrength: GlowStrength
}

const DEFAULT_CONFIG: DesktopLyricsConfig = {
  bgOpacity: 0,
  fontSize: 32,
  colorCurrent: '#FFFFFF',
  colorNext: 'rgba(255,255,255,0.5)',
  glowStrength: 'medium',
}

const STORAGE_KEYS = {
  bgOpacity: 'desktop-lyrics.bg-opacity',
  fontSize: 'desktop-lyrics.font-size',
  colorCurrent: 'desktop-lyrics.color-current',
  colorNext: 'desktop-lyrics.color-next',
  glowStrength: 'desktop-lyrics.glow',
  locked: 'desktop-lyrics.locked',
  posX: 'desktop-lyrics.position-x',
  posY: 'desktop-lyrics.position-y',
} as const

// 锁按钮独立小窗口（永远可点击）—— 仅在歌词窗口上下文管理
const LOCK_WINDOW_LABEL = 'desktop-lyrics-lock'
const LOCK_WINDOW_SIZE = 36
const LOCK_OFFSET_X = 12
const LOCK_OFFSET_Y = 8

// 模块级状态：跨多次 useDesktopLyrics() 调用共享，避免重复注册监听器导致的累积泄漏。
// 参考 usePlayer.ts 的 listenersSetup 模式 + Task 7 useMiniPlayer 改造。
// 调用方：主窗口中 SettingsView + PlayerBar + useGlobalShortcuts（间接），lyrics 窗口、lock 窗口也各自调用。
const visible = ref(false)
const locked = ref(false)
const config = reactive<DesktopLyricsConfig>({ ...DEFAULT_CONFIG })
const currentSong = ref<Song | null>(null)
const unlistens: UnlistenFn[] = []
let stateSyncDone = false
// lyrics 实例只在 lyrics 窗口首次调用时创建（不跨窗口共享，但同窗口内单例）。
// 多个 useDesktopLyrics 实例共享同一个 currentSong ref，所以 lyrics 实例可安全复用。
let lyricsInstance: ReturnType<typeof useLyrics> | null = null

export function useDesktopLyrics() {
  const current = getCurrentWindow()
  const isLyricsWindow = current.label === 'desktop-lyrics'

  // lyrics 窗口首次调用时创建 lyrics 实例；后续调用复用模块级实例。
  if (isLyricsWindow && !lyricsInstance) {
    lyricsInstance = useLyrics(currentSong as Ref<Song | null>)
  }
  const lines = lyricsInstance?.lines ?? ref<LrcLine[]>([])
  const currentLineIndex = lyricsInstance?.currentLineIndex ?? ref(-1)

  // 锁状态初始化与同步 —— 与 setupLyricsWindow 解耦，确保主窗口也能收到锁状态变化
  async function setupStateSync() {
    let initialLocked = false
    try {
      const all = await invoke<Record<string, string>>('get_all_settings')
      initialLocked = all[STORAGE_KEYS.locked] === 'true'
    } catch {
      // 容错：读取失败时按未锁定处理
    }
    locked.value = initialLocked
    if (isLyricsWindow && initialLocked) {
      await current.setIgnoreCursorEvents(true).catch(() => {})
    }

    const un = await listen<boolean>('desktop-lyrics:lock-changed', (e) => {
      locked.value = e.payload
      if (isLyricsWindow) {
        current.setIgnoreCursorEvents(e.payload).catch(() => {})
        updateLockWindowVisibility()
      }
    })
    unlistens.push(un)
  }

  async function syncLockWindowPosition() {
    if (!isLyricsWindow) return
    try {
      const lockWin = await WebviewWindow.getByLabel(LOCK_WINDOW_LABEL)
      if (!lockWin) return
      const pos = await current.outerPosition()
      const size = await current.outerSize()
      const monitor = await currentMonitor()
      const scale = monitor?.scaleFactor ?? 1
      const lockX = Math.round(pos.x + size.width - (LOCK_WINDOW_SIZE + LOCK_OFFSET_X) * scale)
      const lockY = Math.round(pos.y + LOCK_OFFSET_Y * scale)
      await lockWin.setPosition(new PhysicalPosition(lockX, lockY))
    } catch {
      // 静默失败：窗口可能尚未就绪
    }
  }

  async function updateLockWindowVisibility() {
    if (!isLyricsWindow) return
    try {
      const lockWin = await WebviewWindow.getByLabel(LOCK_WINDOW_LABEL)
      if (!lockWin) return
      const lyricsVisible = await current.isVisible()
      if (locked.value && lyricsVisible) {
        await syncLockWindowPosition()
        await lockWin.show()
        await lockWin.setFocus()
      } else {
        await lockWin.hide()
      }
    } catch {
      // 静默失败
    }
  }

  async function setupLyricsWindow(win: Window) {
    // 初始拉取当前歌曲
    try {
      const state = await invoke<{ currentSong: Song | null } | null>('player_get_state')
      if (state?.currentSong) currentSong.value = state.currentSong
    } catch {
      // 容错：未在播放时无 currentSong，useLyrics watch 会处理 null
    }

    // 订阅歌曲切换
    const un1 = await listen<Song | null>('audio:track_change', (e) => {
      currentSong.value = e.payload
    })
    unlistens.push(un1)

    // 位置持久化（debounce 500ms）—— 存 LOGICAL 坐标，避免 physical/logical 混用导致每次启动翻倍
    let moveTimer: ReturnType<typeof setTimeout> | null = null
    const un2 = await listen('tauri://move', () => {
      // 实时跟随：锁窗口位置同步（无 debounce，拖动时立即响应）
      syncLockWindowPosition()
      if (moveTimer) clearTimeout(moveTimer)
      moveTimer = setTimeout(async () => {
        try {
          const pos = await win.outerPosition()
          const monitor = await currentMonitor()
          const scale = monitor?.scaleFactor ?? 1
          // outerPosition 返回物理坐标，除以 scale 转逻辑坐标后存储
          const logicalX = Math.round(pos.x / scale)
          const logicalY = Math.round(pos.y / scale)
          // 合并 posX/posY 为单次 IPC，避免拖动后的串行往返
          await invoke('set_window_position', {
            keyX: STORAGE_KEYS.posX,
            keyY: STORAGE_KEYS.posY,
            valueX: String(logicalX),
            valueY: String(logicalY),
          })
        } catch {
          // 静默失败
        }
      }, 500)
    })
    unlistens.push(un2)

    // 接收主窗口配置同步
    const un3 = await listen<{ key: keyof DesktopLyricsConfig; value: any }>(
      'desktop-lyrics:config-changed',
      (e) => {
        ;(config as any)[e.payload.key] = e.payload.value
      },
    )
    unlistens.push(un3)

    // 监听歌词窗口可见性变化（主窗口 toggle 调用时通知）—— 用于同步锁窗口显示/隐藏
    const un5 = await listen('desktop-lyrics:visibility-changed', () => {
      updateLockWindowVisibility()
    })
    unlistens.push(un5)
  }

  async function toggleLock(value?: boolean) {
    // value 未传时反转（LockBadge 调用）；传入时直接使用（SettingsView checkbox 调用）
    locked.value = value ?? !locked.value
    if (isLyricsWindow) {
      await current.setIgnoreCursorEvents(locked.value)
    }
    await invoke('set_setting', { key: STORAGE_KEYS.locked, value: String(locked.value) })
    // 通知另一侧窗口（主窗口 SettingsView 同步复选框 / 歌词窗口同步视觉态）
    await emit('desktop-lyrics:lock-changed', locked.value)
    if (isLyricsWindow) {
      await updateLockWindowVisibility()
    }
  }

  async function toggle() {
    const target = isLyricsWindow ? current : await WebviewWindow.getByLabel('desktop-lyrics')
    if (!target) return
    if (await target.isVisible()) {
      await target.hide()
      visible.value = false
    } else {
      await target.show()
      await target.setFocus()
      visible.value = true
    }
    // 歌词窗口可见性变化时，同步锁窗口（直接处理或通知歌词窗口处理）
    if (isLyricsWindow) {
      await updateLockWindowVisibility()
    } else {
      await emit('desktop-lyrics:visibility-changed')
    }
  }

  async function updateConfig(key: keyof DesktopLyricsConfig, value: any) {
    ;(config as any)[key] = value
    const storageKey = STORAGE_KEYS[key]
    if (storageKey) {
      await invoke('set_setting', { key: storageKey, value: String(value) })
    }
    // 配置变更广播，desktop-lyrics 窗口实时响应
    await emit('desktop-lyrics:config-changed', { key, value })
  }

  async function restoreConfig() {
    const all = await invoke<Record<string, string>>('get_all_settings')
    try {
      if (all[STORAGE_KEYS.bgOpacity]) config.bgOpacity = parseFloat(all[STORAGE_KEYS.bgOpacity])
      if (all[STORAGE_KEYS.fontSize]) config.fontSize = parseInt(all[STORAGE_KEYS.fontSize])
      if (all[STORAGE_KEYS.colorCurrent]) config.colorCurrent = all[STORAGE_KEYS.colorCurrent]
      if (all[STORAGE_KEYS.colorNext]) config.colorNext = all[STORAGE_KEYS.colorNext]
      if (all[STORAGE_KEYS.glowStrength]) {
        config.glowStrength = all[STORAGE_KEYS.glowStrength] as GlowStrength
      }
    } catch {
      // 配置值非法 → 沿用默认值
      console.warn('[desktop-lyrics] 配置值损坏，沿用默认值')
    }
  }

  async function restoreFromSettings() {
    if (!isLyricsWindow) return
    const all = await invoke<Record<string, string>>('get_all_settings')

    // 固定逻辑宽度（与 tauri.conf.json 一致），不读 outerSize 以避免物理/逻辑混用导致尺寸翻倍
    const WIN_LOGICAL_WIDTH = 1000
    const winLogicalHeight = Math.round(config.fontSize * 2.5) + 24

    await current.setSize(new LogicalSize(WIN_LOGICAL_WIDTH, winLogicalHeight))

    // 获取屏幕信息（重试规避启动时 monitor=null 的时序问题）
    let monitor = await currentMonitor()
    for (let i = 0; i < 3 && !monitor; i++) {
      await new Promise((r) => setTimeout(r, 200))
      monitor = await currentMonitor()
    }

    const scale = monitor?.scaleFactor ?? 1
    const screenLogicalWidth = (monitor?.size.width ?? 1920 * scale) / scale
    const screenLogicalHeight = (monitor?.size.height ?? 1080 * scale) / scale

    // 默认位置：水平居中、底部上方 100px
    const defaultX = Math.round((screenLogicalWidth - WIN_LOGICAL_WIDTH) / 2)
    const defaultY = Math.max(0, Math.round(screenLogicalHeight - winLogicalHeight - 100))

    // 边界检查：旧数据可能是物理坐标或前次错误计算结果，越界则丢弃使用默认
    const storedX = all[STORAGE_KEYS.posX] ? parseInt(all[STORAGE_KEYS.posX]) : NaN
    const storedY = all[STORAGE_KEYS.posY] ? parseInt(all[STORAGE_KEYS.posY]) : NaN

    let x = defaultX
    let y = defaultY
    if (!Number.isNaN(storedX) && storedX >= -WIN_LOGICAL_WIDTH && storedX <= screenLogicalWidth) {
      x = storedX
    }
    if (!Number.isNaN(storedY) && storedY >= 0 && storedY <= screenLogicalHeight) {
      y = storedY
    }

    await current.setPosition(new LogicalPosition(x, y))
  }

  // 幂等 init：仅首次调用时跑 setup，避免跨多次 useDesktopLyrics() 实例累积监听器。
  if (!stateSyncDone) {
    stateSyncDone = true
    void setupStateSync()
    void restoreConfig()
    if (isLyricsWindow) {
      void setupLyricsWindow(current)
    }
  }

  // 模块级监听器随 webview 销毁自动清理，无需 onUnmounted。
  // moveTimer 至多 500ms 后自然 expire，webview 销毁时也会被运行时回收。

  return {
    visible,
    locked,
    config,
    currentSong,
    lines,
    currentLineIndex,
    isLyricsWindow,
    toggle,
    toggleLock,
    updateConfig,
    restoreFromSettings,
  }
}
