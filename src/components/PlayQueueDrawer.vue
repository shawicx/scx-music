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
const { queue, currentSong } = storeToRefs(playerStore)
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
    width="520"
    temporary
  >
    <div class="drawer-inner">
      <div class="drawer-header">
        <v-btn variant="text" size="small" class="mode-btn" @click="cycleMode">
          <v-icon :icon="modeIcon" size="18" />
          <span class="mode-label">{{ modeLabel }}</span>
        </v-btn>
        <v-spacer />
        <span class="drawer-count">{{ t('player.songCount', { count: queue.length }) }}</span>
        <v-btn icon="mdi-close" variant="text" size="small" @click="isOpen = false" />
      </div>

      <div v-if="queue.length === 0" class="empty-queue">
        <p>{{ t('player.noQueue') }}</p>
      </div>

      <div v-else ref="listRef" class="queue-list">
        <div
          v-for="(song, index) in queue"
          :key="song.id"
          class="queue-item"
          :class="{ active: currentSong?.id === song.id }"
          @click="onQueueSongClick(index)"
        >
          <div class="queue-index">
            <v-icon v-if="currentSong?.id === song.id" icon="mdi-play" size="14" color="secondary" />
            <span v-else>{{ index + 1 }}</span>
          </div>
          <div class="queue-song-info">
            <div class="queue-song-title">{{ song.title }}</div>
            <div class="queue-song-artist">{{ song.artist }}</div>
          </div>
          <div class="queue-song-duration">{{ formatTime(song.durationSecs) }}</div>
        </div>
      </div>
    </div>
  </v-navigation-drawer>
</template>

<style scoped>
.drawer-inner {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.drawer-header {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid rgba(var(--v-border-color), 0.12);
}

.mode-btn {
  justify-content: flex-start;
  gap: 8px;
}

.mode-label {
  font-size: var(--text-sm);
}

.drawer-count {
  font-size: var(--text-sm);
  color: var(--v-text-secondary);
  margin-right: 4px;
}

.empty-queue {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 48px 16px;
  color: var(--v-text-secondary);
  font-size: var(--text-md);
}

.queue-list {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
}

.queue-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 16px;
  cursor: pointer;
}

.queue-item:hover {
  background: rgba(var(--v-theme-on-background), 0.04);
}

.queue-item.active {
  background: rgba(var(--v-theme-secondary), 0.08);
}

.queue-item.active .queue-song-title {
  color: rgb(var(--v-theme-secondary));
}

.queue-index {
  width: 24px;
  text-align: center;
  font-size: var(--text-sm);
  color: var(--v-text-secondary);
  flex-shrink: 0;
}

.queue-song-info {
  flex: 1;
  overflow: hidden;
}

.queue-song-title {
  font-size: var(--text-md);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.queue-song-artist {
  font-size: var(--text-sm);
  color: var(--v-text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.queue-song-duration {
  font-size: var(--text-sm);
  color: var(--v-text-secondary);
  flex-shrink: 0;
}
</style>
