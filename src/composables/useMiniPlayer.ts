import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  getCurrentWindow,
  LogicalPosition,
  LogicalSize,
  currentMonitor,
} from '@tauri-apps/api/window'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { listen, emit, type UnlistenFn } from '@tauri-apps/api/event'

const STORAGE_KEYS = {
  active: 'mini-player.active',
  alwaysOnTop: 'mini-player.always-on-top',
  posX: 'mini-player.position-x',
  posY: 'mini-player.position-y',
} as const

const WIN_LOGICAL_WIDTH = 360
const WIN_LOGICAL_HEIGHT = 120
const EDGE_MARGIN = 20

// 模块级共享：跨多个 useMiniPlayer() 实例（主窗口中 App.vue + PlayerBar 都调用）防止并发 toggle
let toggling = false

// 模块级状态：跨多次 useMiniPlayer() 调用共享，避免重复注册监听器导致的累积泄漏。
// 参考 usePlayer.ts 的 listenersSetup 模式 —— 模块级 ref + 幂等 init guard。
const active = ref(false)
const alwaysOnTop = ref(true)
const unlistens: UnlistenFn[] = []
let stateSyncDone = false

async function setupStateSync(
  isMiniPlayerWindow: boolean,
  current: ReturnType<typeof getCurrentWindow>,
) {
  try {
    const all = await invoke<Record<string, string>>('get_all_settings')
    active.value = all[STORAGE_KEYS.active] === 'true'
    alwaysOnTop.value = all[STORAGE_KEYS.alwaysOnTop] !== 'false' // 默认 true
  } catch {
    // 容错：读取失败时沿用默认值
  }

  if (isMiniPlayerWindow) {
    await current.setAlwaysOnTop(alwaysOnTop.value).catch(() => {})
  }

  const un1 = await listen<boolean>('mini-player:active-changed', (e) => {
    active.value = e.payload
  })
  const un2 = await listen<boolean>('mini-player:always-on-top-changed', (e) => {
    alwaysOnTop.value = e.payload
    if (isMiniPlayerWindow) {
      current.setAlwaysOnTop(e.payload).catch(() => {})
    }
  })
  unlistens.push(un1, un2)
}

export function useMiniPlayer() {
  const current = getCurrentWindow()
  const isMiniPlayerWindow = current.label === 'mini-player'

  // moveTimer 保留为函数局部：restoreFromSettings 只在迷你窗口 setup 阶段调用一次，
  // 不存在跨多次 useMiniPlayer() 调用共享的场景。
  let moveTimer: ReturnType<typeof setTimeout> | null = null

  if (!stateSyncDone) {
    stateSyncDone = true
    void setupStateSync(isMiniPlayerWindow, current)
  }

  async function enter(): Promise<boolean> {
    try {
      const main = await WebviewWindow.getByLabel('main')
      const mini = isMiniPlayerWindow ? current : await WebviewWindow.getByLabel('mini-player')
      if (!main || !mini) return false

      // 持久化先于可见性切换：即使后续 show/hide 失败或被 app 关闭打断，
      // DB 状态已经正确，下次启动恢复一致。
      active.value = true
      await invoke('set_setting', { key: STORAGE_KEYS.active, value: 'true' })
      await emit('mini-player:active-changed', true)

      // 顺序很关键：先 show+focus mini，再 hide main。
      // 如果先 hide main，macOS 的 Cmd+M 会最小化整个 app，导致 mini.show() 无法显示。
      await mini.show()
      await mini.setFocus()
      await main.hide()
      return true
    } catch (e) {
      console.error('[mini-player] enter failed:', e)
      return false
    }
  }

  async function exit(): Promise<boolean> {
    try {
      const main = await WebviewWindow.getByLabel('main')
      const mini = isMiniPlayerWindow ? current : await WebviewWindow.getByLabel('mini-player')
      if (!main || !mini) return false

      // 持久化先于可见性切换：用户看到主窗口出现后可能立刻 Cmd+Q，
      // 如果持久化排在 show/hide 后面，IPC 可能来不及完成，导致下次启动误恢复迷你模式。
      active.value = false
      await invoke('set_setting', { key: STORAGE_KEYS.active, value: 'false' })
      await emit('mini-player:active-changed', false)

      await mini.hide()
      await main.show()
      await main.setFocus()
      return true
    } catch (e) {
      console.error('[mini-player] exit failed:', e)
      return false
    }
  }

  async function toggle() {
    if (toggling) return
    toggling = true
    try {
      if (active.value) {
        await exit()
      } else {
        await enter()
      }
    } finally {
      toggling = false
    }
  }

  async function toggleAlwaysOnTop() {
    alwaysOnTop.value = !alwaysOnTop.value
    if (isMiniPlayerWindow) {
      await current.setAlwaysOnTop(alwaysOnTop.value).catch(() => {})
    }
    await invoke('set_setting', { key: STORAGE_KEYS.alwaysOnTop, value: String(alwaysOnTop.value) })
    await emit('mini-player:always-on-top-changed', alwaysOnTop.value)
  }

  async function restoreFromSettings() {
    if (!isMiniPlayerWindow) return

    await current.setSize(new LogicalSize(WIN_LOGICAL_WIDTH, WIN_LOGICAL_HEIGHT))

    let monitor = await currentMonitor()
    for (let i = 0; i < 3 && !monitor; i++) {
      await new Promise((r) => setTimeout(r, 200))
      monitor = await currentMonitor()
    }

    const scale = monitor?.scaleFactor ?? 1
    const screenLogicalWidth = (monitor?.size.width ?? 1920 * scale) / scale
    const screenLogicalHeight = (monitor?.size.height ?? 1080 * scale) / scale

    const defaultX = Math.round(screenLogicalWidth - WIN_LOGICAL_WIDTH - EDGE_MARGIN)
    const defaultY = Math.round(screenLogicalHeight - WIN_LOGICAL_HEIGHT - EDGE_MARGIN)

    const all = await invoke<Record<string, string>>('get_all_settings')
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

    // 位置持久化（debounce 500ms，存逻辑坐标）
    const un = await current.onMoved(async () => {
      if (moveTimer) clearTimeout(moveTimer)
      moveTimer = setTimeout(async () => {
        try {
          const pos = await current.outerPosition()
          const m = await currentMonitor()
          const sf = m?.scaleFactor ?? 1
          const logicalX = Math.round(pos.x / sf)
          const logicalY = Math.round(pos.y / sf)
          // 合并 posX/posY 为单次 IPC,避免拖动后的串行往返
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
    unlistens.push(un)
  }

  // 模块级监听器随 webview 销毁自动清理，无需 onUnmounted。
  // moveTimer 至多 500ms 后自然 expire，webview 销毁时也会被运行时回收。

  return {
    active,
    alwaysOnTop,
    isMiniPlayerWindow,
    enter,
    exit,
    toggle,
    toggleAlwaysOnTop,
    restoreFromSettings,
  }
}
