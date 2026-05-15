<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useTheme } from './composables/useTheme'
import { useLibrary } from './composables/useLibrary'
import { usePlayer } from './composables/usePlayer'
import AppSidebar from './components/AppSidebar.vue'
import LibraryView from './components/LibraryView.vue'
import SettingsView from './components/SettingsView.vue'
import PlayerBar from './components/PlayerBar.vue'
import NowPlayingOverlay from './components/NowPlayingOverlay.vue'

const { loadThemeFromDb } = useTheme()
const { loadFromDb } = useLibrary()
const { toastMsg, toastVisible } = usePlayer()

onMounted(async () => {
  await Promise.all([loadThemeFromDb(), loadFromDb()])
})

const activeView = ref('library')
const showNowPlaying = ref(false)
</script>

<template>
  <v-app>
    <div class="app-shell">
      <AppSidebar :active-view="activeView" @navigate="activeView = $event" />
      <div class="main-area">
        <SettingsView v-if="activeView === 'settings'" @back="activeView = 'library'" />
        <template v-else>
          <LibraryView />
          <PlayerBar @expand="showNowPlaying = true" />
          <Transition name="overlay">
            <NowPlayingOverlay
              v-if="showNowPlaying"
              @close="showNowPlaying = false"
            />
          </Transition>
        </template>
      </div>
    </div>
    <v-snackbar
      v-model="toastVisible"
      :timeout="3000"
      color="error"
      contained
      location="bottom right"
      density="compact"
      rounded="lg"
      elevation="4"
    >
      {{ toastMsg }}
    </v-snackbar>
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

.overlay-enter-active { animation: overlay-fade 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.overlay-leave-active { animation: overlay-fade 0.25s ease-in reverse; }
@keyframes overlay-fade {
  from { opacity: 0; }
  to { opacity: 1; }
}
</style>
