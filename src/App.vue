<script setup lang="ts">
import { ref, watch, onMounted, defineAsyncComponent } from 'vue'
import { onKeyStroke } from '@vueuse/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'
import { useSettingsStore } from './stores/settings'
import { useLibraryStore } from './stores/library'
import { usePlayerStore } from './stores/player'
import { useAudioEffectsStore } from './stores/audioEffects'
import AppSidebar from './components/AppSidebar.vue'
import AppTitlebar from './components/AppTitlebar.vue'
const LibraryView = defineAsyncComponent(() => import('./components/LibraryView.vue'))
const SettingsView = defineAsyncComponent(() => import('./components/SettingsView.vue'))
const AnalysisView = defineAsyncComponent(() => import('./components/AnalysisView.vue'))
const StatsView = defineAsyncComponent(() => import('./components/StatsView.vue'))
const PlayerBar = defineAsyncComponent(() => import('./components/PlayerBar.vue'))
const NowPlayingOverlay = defineAsyncComponent(() => import('./components/NowPlayingOverlay.vue'))
const PlayQueueDrawer = defineAsyncComponent(() => import('./components/PlayQueueDrawer.vue'))
import { useToast } from './composables/useToast'
import { useI18n } from './composables/useI18n'
import { usePageTransition } from './composables/usePageTransition'
import { usePlayerExpand } from './composables/usePlayerExpand'
import { useAutoUpdate } from './composables/useAutoUpdate'
import { useMiniPlayer } from './composables/useMiniPlayer'
import { useGlobalShortcuts } from './composables/useGlobalShortcuts'
import { startCoverAccentWatcher } from './composables/useTheme'
import { isRestoreEnabled, rebuildSourceList, parsePosition } from './composables/useStartupOptions'
import UpdateDialog from './components/UpdateDialog.vue'

const settingsStore = useSettingsStore()
const libraryStore = useLibraryStore()
const playerStore = usePlayerStore()
const audioEffectsStore = useAudioEffectsStore()
const { toastMessage, toastVisible, toastColor } = useToast()
const { initLocale } = useI18n()
const { onEnter: onPageEnter, onLeave: onPageLeave } = usePageTransition()
const { onEnter: onOverlayEnter, onLeave: onOverlayLeave } = usePlayerExpand()
const { startCheck } = useAutoUpdate()
const { enter: enterMini } = useMiniPlayer()
const { init: initGlobalShortcuts } = useGlobalShortcuts()

// 自绘标题栏只在 macOS 且系统真正吃进 titleBarStyle: Overlay 时显示；
// Windows（UA 判断）与 Overlay 被系统忽略的情况（get_titlebar_inset_height 探测）保留原生标题栏
const isMac = navigator.userAgent.includes('Mac')

function isEditable(e: Event) {
  const el = e.target as HTMLElement
  const tag = el.tagName
  return tag === 'INPUT' || tag === 'TEXTAREA' || el.isContentEditable
}

onKeyStroke(' ', (e) => {
  if (isEditable(e)) return
  e.preventDefault()
  playerStore.togglePlayPause()
})

onKeyStroke('ArrowRight', (e) => {
  if (isEditable(e)) return
  playerStore.seekRelative(5)
})

onKeyStroke('ArrowLeft', (e) => {
  if (isEditable(e)) return
  playerStore.seekRelative(-5)
})

onKeyStroke('ArrowUp', (e) => {
  if (isEditable(e)) return
  e.preventDefault()
  playerStore.adjustVolume(0.05)
})

onKeyStroke('ArrowDown', (e) => {
  if (isEditable(e)) return
  e.preventDefault()
  playerStore.adjustVolume(-0.05)
})

onKeyStroke('m', (e) => {
  if (isEditable(e)) return
  // 用 Shift 修饰避开 macOS 默认 Window→Minimize 加速器（Cmd+M）
  if ((e.metaKey || e.ctrlKey) && e.shiftKey) {
    e.preventDefault()
    enterMini()
  }
})

onMounted(async () => {
  const t0 = performance.now()
  // 原生标题栏探测先行：决定 44px 自绘标题行去留，避免布局闪跳
  try {
    const inset = await invoke<number>('get_titlebar_inset_height')
    hasNativeMacBar.value = inset > 1
  } catch {
    // 探测失败按无原生栏处理（Overlay 正常工作的默认形态）
  }
  if (hasNativeMacBar.value) {
    // 原生栏在场：清掉窗口标题文字，应用名只留侧栏 brand 一处
    // （document.title 一并清空，防止 wry 把 webview 标题同步回窗口标题）
    try {
      await getCurrentWindow().setTitle('')
    } catch {
      /* 忽略 */
    }
    document.title = ''
  }
  await initLocale()

  // Run all init tasks concurrently — bootstrap command fetches everything in one IPC
  await Promise.all([
    settingsStore.loadThemeFromDb(),
    libraryStore.loadFromDb(),
    playerStore.setupListeners().then(() => playerStore.getState()),
    initGlobalShortcuts(),
    // 音效恢复：在启动恢复播放之前应用，首播即带 EQ
    audioEffectsStore.init(),
  ])
  // 封面动态取色：跟随当前歌曲写 --accent（依赖主题与播放器就绪）
  void startCoverAccentWatcher()

  console.log(`[perf] App initialized in ${(performance.now() - t0).toFixed(0)}ms`)

  startCheck()

  // 启动恢复播放（若开启）：重建队列 → playFromQueue → seek → 立即暂停
  try {
    const restoreEnabled = await isRestoreEnabled()
    const settings = await invoke<Record<string, string>>('get_all_settings')
    const lastSongId = settings['currentSongId']
    if (restoreEnabled && lastSongId) {
      const pid = settings['activePlaylistId'] ?? null
      const sourceList = rebuildSourceList(
        libraryStore.songs,
        libraryStore.playlistSongs,
        pid,
      )
      const idx = sourceList.findIndex((s) => s.id === lastSongId)
      if (idx >= 0) {
        // playFromQueue 内部会调 player_set_queue → 自动开始播放
        await playerStore.playFromQueue(sourceList, idx)
        // seek 到上次位置（parsePosition 自带非法值/越界钳制）
        const lastPos = parsePosition(settings['last_position'] ?? '', playerStore.duration)
        if (lastPos > 0) {
          await playerStore.seek(lastPos)
        }
        // 直接调 Rust player_pause，避免前端 isPlaying 竞态（事件异步更新）
        await invoke('player_pause')
      }
    }
  } catch (e) {
    // 恢复失败不应阻塞启动，静默降级
    console.warn('[startup] restore playback failed:', e)
  }

  // 启动恢复迷你模式：根据上次状态决定显示哪个窗口
  // 主窗口默认 visible:false（tauri.conf.json），必须显式 show
  try {
    const settings = await invoke<Record<string, string>>('get_all_settings')
    if (settings['mini-player.active'] === 'true') {
      const ok = await enterMini()
      if (!ok) {
        // mini 窗口未就绪等失败情况：重置 DB 并显示主窗口，避免两边都隐藏
        await invoke('set_setting', { key: 'mini-player.active', value: 'false' }).catch(() => {})
        await getCurrentWindow().show()
      }
    } else {
      await getCurrentWindow().show()
    }
  } catch {
    // 容错：失败时确保主窗口可见
    await getCurrentWindow().show().catch(() => {})
  }
})

const activeView = ref('library')
const showNowPlaying = ref(false)
// 玻璃质感：np 打开期间隐藏主界面表面并让窗口链路透明（桌面经原生毛玻璃透出），
// after-leave 后才恢复，避免退场动画期间表面闪回
const npVisible = ref(false)
const showQueue = ref(false)
const sbWidth = ref(236)
watch(npVisible, (v) => {
  document.body.classList.toggle('np-glass', v)
})
watch(showNowPlaying, (v) => {
  if (v) npVisible.value = true
})
// 系统忽略了 titleBarStyle: Overlay（如 macOS 26 beta）时原生标题栏占位，
// 此时收起自绘标题栏，避免双标题栏（探测命令由 Rust 侧 NSWindow 几何计算）
const hasNativeMacBar = ref(false)
</script>

<template>
  <v-app>
    <div
      class="app-shell"
      :class="{ 'no-titlebar': !isMac || hasNativeMacBar, 'native-mac-bar': hasNativeMacBar }"
      :style="{ '--sb-w': sbWidth + 'px' }"
    >
      <AppTitlebar v-if="isMac && !hasNativeMacBar" class="app-titlebar" />
      <AppSidebar v-show="!npVisible" class="app-sidebar" :active-view="activeView" @resize="sbWidth = $event" @navigate="activeView = $event" />
      <main v-show="!npVisible" class="app-main">
        <Transition :css="false" mode="out-in" @enter="onPageEnter" @leave="onPageLeave">
          <SettingsView v-if="activeView === 'settings'" key="settings" @back="activeView = 'library'" />
          <AnalysisView v-else-if="activeView === 'analysis'" key="analysis" @back="activeView = 'library'" />
          <StatsView v-else-if="activeView === 'stats'" key="stats" @back="activeView = 'library'" />
          <LibraryView v-else key="library" />
        </Transition>
      </main>
      <PlayerBar
        v-show="!npVisible"
        class="app-playerbar"
        @expand="showNowPlaying = true"
        @toggle-queue="showQueue = !showQueue"
      />
      <Transition :css="false" @enter="onOverlayEnter" @leave="onOverlayLeave" @after-leave="npVisible = false">
        <NowPlayingOverlay
          v-if="showNowPlaying"
          @close="showNowPlaying = false"
          @toggle-queue="showQueue = !showQueue"
        />
      </Transition>
    </div>
    <PlayQueueDrawer v-model="showQueue" />
    <v-snackbar
      v-model="toastVisible"
      :color="toastColor"
      :timeout="3000"
      contained
      location="bottom"
      offset="76"
      density="compact"
      rounded="pill"
      class="app-toast"
      elevation="0"
    >
      {{ toastMessage }}
    </v-snackbar>
    <UpdateDialog />
  </v-app>
</template>

<style>
*, *::before, *::after { box-sizing: border-box; }
:root {
  --text-xs: 10px;
  --text-sm: 11px;
  --text-md: 12.5px;
  --text-lg: 13px;
  --text-xl: 17px;
  --text-2xl: 21px;
}
html, body, #app {
  height: 100%; overflow: hidden;
  font-family: var(--font-ui);
  font-size: 100%; /* 固定基准，组件按 px 排版 */
}
::selection { background: rgb(var(--accent-rgb) / 0.32); }
::-webkit-scrollbar { width: 8px; height: 8px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb {
  background: var(--hairline-strong);
  border-radius: var(--radius-pill);
  border: 2px solid transparent;
  background-clip: content-box;
}

/* Tooltip 皮肤：玻璃胶囊（匹配浮层玻璃语言） */
.v-tooltip .v-overlay__content.scx-tip {
  background: var(--glass-bg);
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
  border: 1px solid var(--glass-border);
  border-radius: var(--radius-sm);
  font-size: var(--text-sm);
  letter-spacing: 0.02em;
  padding: 5px 10px;
  color: var(--text-1);
  box-shadow: var(--shadow-pop);
}

/* Toast：玻璃胶囊，底部居中 */
.v-snackbar.v-overlay .v-snackbar__content {
  padding: 0;
}
.app-toast .v-snackbar__wrapper {
  background: var(--glass-bg);
  backdrop-filter: blur(22px);
  -webkit-backdrop-filter: blur(22px);
  border: 1px solid var(--glass-border);
  color: var(--text-1);
  box-shadow: var(--shadow-pop);
  font-size: var(--text-md);
  min-width: 0;
  padding: 8px 18px;
}
</style>

<style scoped>
/* 主窗口背景/前景色:限定 scoped,避免泄漏到子窗口(桌面歌词/mini)导致透明失效 */
.app-shell {
  height: 100vh;
  position: relative;
  display: grid;
  grid-template-columns: var(--sb-w) 1fr;
  grid-template-rows: 44px 1fr 64px;
  grid-template-areas:
    "tb tb"
    "sb main"
    "pb pb";
  background: var(--surface-1);
  color: var(--text-1);
  font-size: 13px;
  line-height: 1.5;
  transition: background 0.35s var(--ease), color 0.35s var(--ease);
}
/* Windows / Overlay 失效（系统忽略 titleBarStyle）：保留原生标题栏，收起自绘标题行 */
.app-shell.no-titlebar {
  grid-template-rows: 0 1fr 64px;
}
/* 玻璃质感正在播放：整链透明，桌面经原生毛玻璃透出（表面已 v-show 隐藏） */
:global(body.np-glass) .app-shell {
  background: transparent;
}
:global(body.np-glass) .v-application {
  background: transparent;
}
.app-titlebar { grid-area: tb; }
.app-sidebar { grid-area: sb; min-width: 0; }
.app-main {
  grid-area: main;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  position: relative;
}
.app-playerbar { grid-area: pb; position: relative; z-index: 35; }
</style>
