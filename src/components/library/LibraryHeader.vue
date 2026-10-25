<script setup lang="ts">
import { computed } from 'vue'
import type { DisplayMode, SortBy, SortOrder } from '../../types'
import SegControl from '../common/SegControl.vue'
import { useI18n } from '../../composables/useI18n'

const props = defineProps<{
  pageTitle: string
  pageSubtitle: string
  songCount: number
  totalDurationSecs: number
  searchQuery: string
  displayMode: DisplayMode
  viewMode: 'list' | 'grid'
  sortBy: SortBy
  sortOrder: SortOrder
  showSortOption?: boolean
}>()

const emit = defineEmits<{
  'update:searchQuery': [value: string]
  'update:displayMode': [mode: DisplayMode]
  'update:viewMode': [mode: 'list' | 'grid']
  'openSortMenu': [event: MouseEvent]
  'back': []
}>()

const { t } = useI18n()

const metaText = computed(() => {
  const count = t('common.songs', { count: props.songCount })
  if (props.totalDurationSecs > 0) {
    return `${count} · ${t('library.approxMinutes', { minutes: Math.round(props.totalDurationSecs / 60) })}`
  }
  return count
})

const displayOptions = [
  { value: 'songs', label: t('library.viewSongs') },
  { value: 'albums', label: t('library.viewAlbums') },
  { value: 'artists', label: t('library.viewArtists') },
]
</script>

<template>
  <header class="lib-head">
    <div class="lh-row">
      <button v-if="pageSubtitle" class="lh-back" :aria-label="t('common.back')" @click="emit('back')">
        <v-icon icon="mdi-chevron-left" size="18"></v-icon>
      </button>
      <h1 class="lh-title">{{ pageTitle }}</h1>
      <div class="lh-meta num">{{ metaText }}</div>
      <div class="lh-tools">
        <div class="search">
          <v-icon icon="mdi-magnify" size="14"></v-icon>
          <input
            type="search"
            :value="searchQuery"
            :placeholder="t('library.searchSongs')"
            :aria-label="t('library.searchSongs')"
            @input="emit('update:searchQuery', ($event.target as HTMLInputElement).value)"
          />
        </div>
        <SegControl
          :options="displayOptions"
          :model-value="displayMode"
          :aria-label="t('library.browseMode')"
          @update:model-value="emit('update:displayMode', $event as DisplayMode)"
        />
        <SegControl
          v-if="showSortOption"
          :options="[
            { value: 'list', icon: 'mdi-view-list', iconSize: 15 },
            { value: 'grid', icon: 'mdi-view-grid', iconSize: 15 },
          ]"
          :model-value="viewMode"
          :aria-label="t('library.viewMode')"
          @update:model-value="emit('update:viewMode', $event as 'list' | 'grid')"
        />
        <button
          class="lh-sort icon-btn"
          :aria-label="t('library.sort')"
          @click="emit('openSortMenu', $event)"
        >
          <v-icon icon="mdi-sort-variant" size="16"></v-icon>
        </button>
      </div>
    </div>
    <div class="lh-rule"></div>
  </header>
</template>

<style scoped>
.lib-head {
  padding: 20px 28px 0;
  flex-shrink: 0;
}
.lh-row {
  display: flex;
  align-items: flex-end;
  gap: 16px;
  min-height: 52px;
}
.lh-back {
  width: 30px;
  height: 30px;
  border-radius: 8px;
  margin-bottom: 5px;
  display: grid;
  place-items: center;
  color: var(--text-2);
  flex-shrink: 0;
  background: none;
  border: none;
  cursor: pointer;
}
.lh-back:hover {
  background: var(--surface-2);
  color: var(--text-1);
}
.lh-title {
  font-family: var(--font-display);
  font-size: clamp(24px, 2.6vw, 30px);
  font-weight: 560;
  line-height: 1.15;
  letter-spacing: 0.01em;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.lh-meta {
  font-size: 12px;
  color: var(--text-3);
  padding-bottom: 6px;
  white-space: nowrap;
}
.lh-tools {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 8px;
  padding-bottom: 3px;
  flex-wrap: wrap;
  justify-content: flex-end;
}

.search {
  display: flex;
  align-items: center;
  gap: 7px;
  min-width: 180px;
  max-width: 230px;
  flex: 1 1 170px;
  padding: 6px 10px;
  border-radius: 8px;
  background: var(--surface-2);
  border: var(--hairline-w) solid transparent;
  transition: border-color 0.18s, background 0.18s;
  color: var(--text-3);
}
.search:focus-within {
  border-color: rgb(var(--accent-rgb) / 0.55);
}
.search input {
  flex: 1;
  min-width: 0;
  border: none;
  background: none;
  outline: none;
  color: var(--text-1);
  font: inherit;
  font-size: 12.5px;
}
.search input::placeholder {
  color: var(--text-3);
}

.icon-btn {
  width: 30px;
  height: 30px;
  border-radius: 8px;
  display: grid;
  place-items: center;
  color: var(--text-2);
  transition: background 0.14s, color 0.14s, transform 0.1s;
  flex-shrink: 0;
  background: none;
  border: none;
  cursor: pointer;
}
.icon-btn:hover {
  background: var(--surface-2);
  color: var(--text-1);
}
.icon-btn:active {
  transform: scale(0.94);
}

.lh-rule {
  height: var(--hairline-w);
  background: var(--hairline);
  margin-top: 14px;
}

@media (max-width: 1100px) {
  .search {
    min-width: 140px;
  }
  th.c-album,
  td.c-album {
    display: none;
  }
}
</style>
