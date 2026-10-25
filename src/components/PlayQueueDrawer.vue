<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { usePlayerStore } from '../stores/player'
import { usePlaybackMode } from '../composables/usePlaybackMode'
import { useI18n } from '../composables/useI18n'
import { useAnimation } from '../composables/useAnimation'

const props = defineProps<{ modelValue: boolean }>()
const emit = defineEmits<{ 'update:modelValue': [value: boolean] }>()

const playerStore = usePlayerStore()
const { queue, currentSong, isPlaying } = storeToRefs(playerStore)
const { formatTime } = playerStore
const { modeIcon, modeLabel, cycleMode } = usePlaybackMode()
const { t } = useI18n()
const { Flip, easings } = useAnimation()

const listRef = ref<HTMLElement | null>(null)

const isOpen = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val),
})

function onQueueSongClick(index: number) {
  playerStore.playFromQueue(queue.value, index)
}

watch(queue, (newQueue, oldQueue) => {
  if (!listRef.value || !oldQueue) return
  const sameIds = new Set(newQueue.map(s => s.id))
  const oldIds = new Set(oldQueue.map(s => s.id))
  if (sameIds.size !== oldIds.size) return

  const state = Flip.getState(listRef.value.querySelectorAll('.queue-item'))
  // Wait for DOM update with reordered items
  requestAnimationFrame(() => {
    Flip.from(state, {
      duration: 0.35,
      ease: easings.fluid,
      absolute: true,
    })
  })
})
</script>

<template>
  <v-navigation-drawer
    v-model="isOpen"
    location="right"
    width="340"
    temporary
    class="queue-drawer"
  >
    <div class="drawer-inner" :class="{ 'is-playing': isPlaying }">
      <div class="q-head">
        <div class="q-head-text">
          <div class="q-title">{{ t('player.playQueue') }}</div>
          <div class="q-count num">{{ t('player.songCount', { count: queue.length }) }}</div>
        </div>
        <div class="q-head-actions">
          <button class="q-icon-btn" :aria-label="modeLabel" :title="modeLabel" @click="cycleMode">
            <v-icon :icon="modeIcon" size="16"></v-icon>
          </button>
          <button class="q-icon-btn" :aria-label="t('player.collapse')" @click="isOpen = false">
            <v-icon icon="mdi-close" size="16"></v-icon>
          </button>
        </div>
      </div>

      <div v-if="queue.length === 0" class="q-empty">
        <p>{{ t('player.noQueue') }}</p>
      </div>

      <div v-else ref="listRef" class="q-list">
        <div
          v-for="(song, index) in queue"
          :key="song.id"
          class="queue-item"
          :class="{ current: currentSong?.id === song.id }"
          @click="onQueueSongClick(index)"
        >
          <span class="qi num">
            <span v-if="currentSong?.id === song.id" class="eq" aria-hidden="true"><i></i><i></i><i></i></span>
            <span v-else>{{ index + 1 }}</span>
          </span>
          <span class="qm">
            <span class="qt">{{ song.title }}</span>
            <span class="qa">{{ song.artist }}</span>
          </span>
          <span class="qd num">{{ formatTime(song.durationSecs) }}</span>
        </div>
      </div>
    </div>
  </v-navigation-drawer>
</template>

<style src="../styles/eq-indicator.css"></style>

<style scoped>
/* v-navigation-drawer 容器：玻璃化 */
.drawer-inner {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--glass-bg);
  -webkit-backdrop-filter: blur(30px) saturate(1.5);
  backdrop-filter: blur(30px) saturate(1.5);
  border-left: var(--hairline-w) solid var(--glass-border);
}
:global(.scx-light) .drawer-inner {
  background: var(--surface-3);
  -webkit-backdrop-filter: none;
  backdrop-filter: none;
}

.q-head {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 16px 16px 12px 20px;
  border-bottom: var(--hairline-w) solid var(--hairline);
}
.q-head-text {
  min-width: 0;
}
.q-title {
  font-family: var(--font-display);
  font-size: 17px;
  font-weight: 560;
  color: var(--text-1);
}
.q-count {
  font-size: 11.5px;
  color: var(--text-3);
  margin-top: 1px;
}
.q-head-actions {
  margin-left: auto;
  display: flex;
  gap: 2px;
}
.q-icon-btn {
  width: 30px;
  height: 30px;
  border-radius: 8px;
  display: grid;
  place-items: center;
  color: var(--text-2);
  background: none;
  border: none;
  cursor: pointer;
  transition: background 0.14s, color 0.14s;
}
.q-icon-btn:hover {
  background: var(--surface-2);
  color: var(--text-1);
}

.q-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  color: var(--text-3);
  font-size: 12.5px;
  text-align: center;
}

.q-list {
  flex: 1;
  overflow-y: auto;
  padding: 6px 8px 14px;
}

.queue-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 7px 10px;
  border-radius: 8px;
  cursor: pointer;
}
.queue-item:hover {
  background: var(--surface-2);
}
.qi {
  font-size: 11px;
  color: var(--text-3);
  width: 20px;
  text-align: right;
  flex-shrink: 0;
}
.qm {
  min-width: 0;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 1px;
}
.qt {
  font-size: 12.5px;
  font-weight: 560;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.qa {
  font-size: 11px;
  color: var(--text-3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.qd {
  font-size: 11px;
  color: var(--text-3);
  flex-shrink: 0;
}
.queue-item.current .qt,
.queue-item.current .qi {
  color: var(--accent);
}
</style>
