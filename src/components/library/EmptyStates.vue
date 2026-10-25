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
  <div class="empty">
    <v-icon :icon="stateConfig[type].icon" size="40" class="e-icon"></v-icon>
    <div class="e-title">{{ t(stateConfig[type].titleKey) }}</div>
    <p>{{ t(stateConfig[type].hintKey) }}</p>
  </div>
</template>

<style scoped>
.empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 64px 20px;
  color: var(--text-3);
  gap: 0;
}
.e-icon {
  opacity: 0.4;
  margin-bottom: 12px;
}
.e-title {
  font-family: var(--font-display);
  font-size: 21px;
  color: var(--text-2);
  font-weight: 540;
}
.empty p {
  font-size: 12.5px;
  margin-top: 6px;
}
</style>
