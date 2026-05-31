<script setup lang="ts">
import { useI18n } from '../../composables/useI18n'

defineProps<{
  type: 'noPlaylist' | 'emptyPlaylist' | 'noResults'
}>()

const { t } = useI18n()

const stateConfig: Record<string, { icon: string; titleKey: string; hintKey: string }> = {
  noPlaylist: {
    icon: 'mdi-playlist-music',
    titleKey: 'empty.noPlaylistTitle',
    hintKey: 'empty.noPlaylistHint',
  },
  emptyPlaylist: {
    icon: 'mdi-music',
    titleKey: 'empty.emptyPlaylistTitle',
    hintKey: 'empty.emptyPlaylistHint',
  },
  noResults: {
    icon: 'mdi-magnify',
    titleKey: 'empty.noResultsTitle',
    hintKey: 'empty.noResultsHint',
  },
}
</script>

<template>
  <div class="empty-state">
    <v-icon :icon="stateConfig[type].icon" size="48" color="secondary"></v-icon>
    <p class="empty-text">{{ t(stateConfig[type].titleKey) }}</p>
    <p class="empty-hint">{{ t(stateConfig[type].hintKey) }}</p>
  </div>
</template>

<style scoped>
.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  opacity: 0.5;
}

.empty-text {
  font-size: var(--text-lg);
  color: var(--v-text-secondary);
}

.empty-hint {
  font-size: var(--text-sm);
  color: var(--v-text-muted);
}
</style>
