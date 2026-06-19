import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { usePlayerStore } from '../stores/player'
import { useMiniPlayer } from './useMiniPlayer'
import { useDesktopLyrics } from './useDesktopLyrics'
import { usePlaybackMode } from './usePlaybackMode'

interface ShortcutDefault {
  id: string
  combo: string
  enabled: boolean
}

type ActionHandler = () => void | Promise<void>

// 常量与 settings-key helper
const SETTINGS_PREFIX = 'shortcut'
const comboKey = (id: string) => `${SETTINGS_PREFIX}.${id}`
const enabledKey = (id: string) => `${SETTINGS_PREFIX}.${id}.enabled`
const MUTED_VOLUME_KEY = `${SETTINGS_PREFIX}.muted-volume`

// 模块级单例状态（所有 useGlobalShortcuts() 调用方共享）
let unlisten: UnlistenFn | null = null
let defaultsCache: Map<string, ShortcutDefault> = new Map()
let initPromise: Promise<void> | null = null
let mutedVolume: number | null = null

/** 设置 mutedVolume 并持久化到 settings */
async function setMutedVolume(v: number | null) {
  mutedVolume = v
  await invoke('set_setting', { key: MUTED_VOLUME_KEY, value: v === null ? '' : String(v) }).catch(() => {})
}

/**
 * Action ID → handler 工厂
 * 工厂模式：每次触发时解析 store/composable，避免模块加载时 Pinia 还未初始化
 */
const ACTION_HANDLERS: Record<string, () => ActionHandler> = {
  'media.play-pause':   () => usePlayerStore().togglePlayPause,
  'media.next':         () => usePlayerStore().next,
  'media.previous':     () => usePlayerStore().previous,
  'media.stop':         () => usePlayerStore().stop,
  'media.volume-up': () => async () => {
    usePlayerStore().adjustVolume(0.05)
    if (mutedVolume !== null) await setMutedVolume(null)
  },
  'media.volume-down': () => async () => {
    usePlayerStore().adjustVolume(-0.05)
    if (mutedVolume !== null) await setMutedVolume(null)
  },
  'media.mute': () => async () => {
    const s = usePlayerStore()
    if (mutedVolume !== null) {
      s.setVolume(mutedVolume)
      await setMutedVolume(null)
    } else {
      await setMutedVolume(s.volume)
      s.setVolume(0)
    }
  },
  'app.mini-player': () => {
    const { enter, exit, active } = useMiniPlayer()
    return () => (active.value ? exit() : enter())
  },
  'app.desktop-lyrics': () => {
    const { toggle } = useDesktopLyrics()
    return toggle
  },
  'app.toggle-window':  () => () => invoke('app_toggle_main_window'),
  'app.cycle-mode':     () => usePlaybackMode().cycleMode,
}

/**
 * 校验：实现前先确认 usePlayerStore 有以下方法
 * - togglePlayPause, next, previous, stop, setVolume, adjustVolume
 */
function verifyPlayerStoreMethods() {
  const s = usePlayerStore()
  const required = ['togglePlayPause', 'next', 'previous', 'stop', 'adjustVolume', 'setVolume']
  for (const m of required) {
    if (typeof (s as any)[m] !== 'function') {
      console.warn(`[shortcuts] usePlayerStore.${m} is missing — action dispatch will fail`)
    }
  }
}

async function doInit() {
  verifyPlayerStoreMethods()

  // 恢复 muted 状态（避免 webview reload 后陷入永久静音）
  // 注意：app restart 时 Rust 音频引擎会重置为默认音量 1.0，
  // 但 mutedVolume 从 DB 恢复后 UI 以为是静音状态，导致短暂"假静音"。
  // 因此恢复 mutedVolume 后需显式将 Rust 音频音量设为 0。
  try {
    const stored0 = await invoke<Record<string, string>>('get_all_settings')
    const muted = stored0[MUTED_VOLUME_KEY]
    if (muted && muted !== '') {
      const v = Number(muted)
      if (!Number.isNaN(v) && v > 0) {
        mutedVolume = v
        // 应用 mute 到 Rust 音频引擎（绕过 playerStore，可能尚未就绪）
        try {
          await invoke('player_set_volume', { volume: 0 })
        } catch (e) {
          console.warn('[shortcuts] failed to apply mute on init:', e)
        }
      }
    }
  } catch {}

  // 1. 拉取默认值清单（用于后续 rebind/resetAll 解析回退值）
  // 注意：启动注册由 Rust setup_shortcuts_at_start 完成，JS 不再重复注册。
  const defaults = await invoke<ShortcutDefault[]>('shortcuts_list_defaults')
  defaultsCache = new Map(defaults.map(d => [d.id, d]))

  // 2. 监听触发事件（单例 — 仅注册一次监听器）
  if (!unlisten) {
    unlisten = await listen<string>('shortcut-triggered', (e) => {
      const factory = ACTION_HANDLERS[e.payload]
      if (!factory) {
        console.warn(`[shortcuts] no handler for action "${e.payload}"`)
        return
      }
      try {
        const handler = factory()
        Promise.resolve(handler()).catch(err => {
          console.error(`[shortcuts] async handler for ${e.payload} threw:`, err)
        })
      } catch (err) {
        console.error(`[shortcuts] handler for ${e.payload} threw:`, err)
      }
    })
  }
}

export function useGlobalShortcuts() {
  async function init() {
    if (initPromise) return initPromise
    initPromise = doInit()
    return initPromise
  }

  /** 解析当前生效组合：DB 值 > 默认值 */
  function resolveCombo(actionId: string, stored: Record<string, string>): string {
    return stored[comboKey(actionId)] ?? defaultsCache.get(actionId)?.combo ?? ''
  }

  /** 事务性 rebind：unregister 旧 → register 新 → 写库；失败回滚 */
  async function rebind(actionId: string, newCombo: string): Promise<{ ok: true } | { ok: false; error: string }> {
    const stored = await invoke<Record<string, string>>('get_all_settings')
    const oldCombo = resolveCombo(actionId, stored)
    try {
      await invoke('shortcuts_unregister', { actionId })
      await invoke('shortcuts_register', { actionId, combo: newCombo })
      await invoke('set_setting', { key: comboKey(actionId), value: newCombo })
      return { ok: true }
    } catch (e: any) {
      // 回滚：重新注册旧组合（如果有）
      if (oldCombo) {
        try { await invoke('shortcuts_register', { actionId, combo: oldCombo }) } catch {}
      }
      return { ok: false, error: e instanceof Error ? e.message : String(e) }
    }
  }

  async function setEnabled(actionId: string, enabled: boolean): Promise<void> {
    if (enabled) {
      const stored = await invoke<Record<string, string>>('get_all_settings')
      const combo = resolveCombo(actionId, stored)
      if (combo) await invoke('shortcuts_register', { actionId, combo })
    } else {
      await invoke('shortcuts_unregister', { actionId })
    }
    await invoke('set_setting', { key: enabledKey(actionId), value: String(enabled) })
  }

  async function isComboRegistered(combo: string): Promise<boolean> {
    try {
      return await invoke<boolean>('shortcuts_is_registered', { combo })
    } catch {
      return false
    }
  }

  /** 重置所有绑定到默认值：注销全部 → 写入默认值 → 按默认重新注册 */
  async function resetAll(): Promise<void> {
    if (defaultsCache.size === 0) return  // M-4: 未初始化时直接返回，避免无谓 IPC
    // 1. 注销所有当前注册的（通过遍历 defaultsCache 反查）
    for (const def of defaultsCache.values()) {
      try { await invoke('shortcuts_unregister', { actionId: def.id }) } catch {}
    }
    // 2. 写入默认值到 DB
    for (const def of defaultsCache.values()) {
      await invoke('set_setting', { key: comboKey(def.id), value: def.combo })
      await invoke('set_setting', { key: enabledKey(def.id), value: String(def.enabled) })
    }
    // 3. 重新注册默认 enabled 的
    const toRegister: [string, string][] = []
    for (const def of defaultsCache.values()) {
      if (def.enabled && def.combo) toRegister.push([def.id, def.combo])
    }
    await invoke('shortcuts_register_all', { bindings: toRegister })
  }

  function destroy() {
    unlisten?.()
    unlisten = null
    initPromise = null  // 允许 destroy 后重新 init
  }

  return { init, rebind, setEnabled, isComboRegistered, resetAll, destroy }
}
