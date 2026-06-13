<script setup lang="ts">
import { ref, onMounted, defineAsyncComponent } from 'vue'
import { onKeyStroke } from '@vueuse/core'
import { useSettingsStore } from './stores/settings'
import { useLibraryStore } from './stores/library'
import { usePlayerStore } from './stores/player'
import AppSidebar from './components/AppSidebar.vue'
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
import UpdateDialog from './components/UpdateDialog.vue'

const settingsStore = useSettingsStore()
const libraryStore = useLibraryStore()
const playerStore = usePlayerStore()
const { toastMessage, toastVisible, toastColor } = useToast()
const { initLocale } = useI18n()
const { onEnter: onPageEnter, onLeave: onPageLeave } = usePageTransition()
const { onEnter: onOverlayEnter, onLeave: onOverlayLeave } = usePlayerExpand()
const { startCheck } = useAutoUpdate()

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

onKeyStroke('MediaTrackNext', () => playerStore.next())
onKeyStroke('MediaTrackPrevious', () => playerStore.previous())

onMounted(async () => {
  const t0 = performance.now()
  await initLocale()

  // Run all init tasks concurrently — bootstrap command fetches everything in one IPC
  await Promise.all([
    settingsStore.loadThemeFromDb(),
    libraryStore.loadFromDb(),
    playerStore.setupListeners().then(() => playerStore.getState()),
  ])

  console.log(`[perf] App initialized in ${(performance.now() - t0).toFixed(0)}ms`)

  startCheck()
})

const activeView = ref('library')
const showNowPlaying = ref(false)
const showQueue = ref(false)
</script>

<template>
  <v-app>
    <div class="app-shell">
      <AppSidebar :active-view="activeView" @navigate="activeView = $event" />
      <div class="main-area">
        <Transition :css="false" mode="out-in" @enter="onPageEnter" @leave="onPageLeave">
          <SettingsView v-if="activeView === 'settings'" key="settings" @back="activeView = 'library'" />
          <AnalysisView v-else-if="activeView === 'analysis'" key="analysis" @back="activeView = 'library'" />
          <StatsView v-else-if="activeView === 'stats'" key="stats" @back="activeView = 'library'" />
          <div v-else key="library" class="library-wrapper">
            <LibraryView />
            <PlayerBar @expand="showNowPlaying = true" @toggle-queue="showQueue = !showQueue" />
            <Transition :css="false" @enter="onOverlayEnter" @leave="onOverlayLeave">
              <NowPlayingOverlay
                v-if="showNowPlaying"
                @close="showNowPlaying = false"
              />
            </Transition>
          </div>
        </Transition>
      </div>
    </div>
    <PlayQueueDrawer v-model="showQueue" />
    <v-snackbar
      v-model="toastVisible"
      :color="toastColor"
      :timeout="3000"
      contained
      location="bottom right"
      density="compact"
      rounded="lg"
      elevation="4"
    >
      {{ toastMessage }}
    </v-snackbar>
    <UpdateDialog />
  </v-app>
</template>

<style>
*, *::before, *::after { box-sizing: border-box; }
:root {
  --text-xs: 0.6rem;
  --text-sm: 0.7rem;
  --text-md: 0.8rem;
  --text-lg: 0.95rem;
  --text-xl: 1.1rem;
  --text-2xl: 1.3rem;
}
html, body, #app {
  height: 100%; overflow: hidden;
  background: rgb(var(--v-theme-background)); color: rgb(var(--v-theme-on-background));
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  font-size: clamp(16px, 1vw + 4px, 24px);
}
::-webkit-scrollbar { width: 6px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: rgb(var(--v-theme-surface-variant)); border-radius: 3px; }
</style>

<style scoped>
.app-shell { display: flex; height: 100vh; position: relative; }
.main-area { flex: 1; display: flex; flex-direction: column; overflow: hidden; position: relative; }
.library-wrapper { display: flex; flex-direction: column; flex: 1; overflow: hidden; }
</style>
