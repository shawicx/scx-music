<script setup lang="ts">
import { computed } from 'vue'
import type { DisplayMode, SortBy, SortOrder } from '../../types'
import { useI18n } from '../../composables/useI18n'

const props = defineProps<{
  pageTitle: string
  pageSubtitle: string
  songCount: number
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

const sortOptions = [
  { value: 'default' as const, labelKey: 'library.sortDefault', icon: 'mdi-sort-variant' },
  { value: 'title' as const, labelKey: 'library.sortByTitle', icon: 'mdi-format-title' },
  { value: 'artist' as const, labelKey: 'library.sortByArtist', icon: 'mdi-account-music' },
  { value: 'album' as const, labelKey: 'library.sortByAlbum', icon: 'mdi-album' },
  { value: 'duration' as const, labelKey: 'library.sortByDuration', icon: 'mdi-clock-outline' },
]

const sortLabel = computed(() => {
  const option = sortOptions.find(o => o.value === props.sortBy)
  if (!option) return t('library.sort')
  const label = t(option.labelKey)
  if (props.sortBy === 'default') return label
  return `${label} ${props.sortOrder === 'asc' ? '↑' : '↓'}`
})
</script>

<template>
  <div class="top-bar">
    <div class="top-bar-left">
      <v-btn v-if="pageSubtitle" icon size="x-small" variant="plain" density="compact" @click="emit('back')">
        <v-icon icon="mdi-chevron-left" size="14"></v-icon>
      </v-btn>
      <div class="title-group">
        <h1 class="page-title">{{ pageTitle }}</h1>
        <span v-if="pageSubtitle" class="page-subtitle">{{ pageSubtitle }}</span>
      </div>
      <v-chip v-if="songCount > 0" size="x-small" variant="flat" color="surface">
        {{ t('common.songs', { count: songCount }) }}
      </v-chip>
    </div>
    <div class="top-bar-right">
      <v-text-field
        :model-value="searchQuery"
        @update:model-value="emit('update:searchQuery', $event)"
        prepend-inner-icon="mdi-magnify"
        :placeholder="t('library.searchSongs')"
        density="compact"
        variant="solo-filled"
        hide-details
        single-line
        bg-color="surface"
        rounded="lg"
        class="search-field"
      />
      <v-btn-toggle :model-value="displayMode" @update:model-value="emit('update:displayMode', $event)" mandatory density="compact" variant="outlined" divided>
        <v-btn value="songs" size="small">
          <v-icon icon="mdi-music" size="16"></v-icon>
        </v-btn>
        <v-btn value="albums" size="small">
          <v-icon icon="mdi-album" size="16"></v-icon>
        </v-btn>
        <v-btn value="artists" size="small">
          <v-icon icon="mdi-microphone-variant" size="16"></v-icon>
        </v-btn>
      </v-btn-toggle>
      <v-btn-toggle v-if="showSortOption" :model-value="viewMode" @update:model-value="emit('update:viewMode', $event)" mandatory density="compact" variant="outlined" divided>
        <v-btn value="list" size="small">
          <v-icon icon="mdi-view-list" size="16"></v-icon>
        </v-btn>
        <v-btn value="grid" size="small">
          <v-icon icon="mdi-view-grid" size="16"></v-icon>
        </v-btn>
      </v-btn-toggle>
      <v-btn variant="outlined" append-icon="mdi-sort-variant" @click="emit('openSortMenu', $event)">
        {{ sortLabel }}
      </v-btn>
    </div>
  </div>
</template>

<style scoped>
.top-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 20px;
  border-bottom: 1px solid var(--v-border-color);
}

.top-bar-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.title-group {
  display: flex;
  align-items: baseline;
  gap: 8px;
}

.page-title {
  font-size: var(--text-lg);
  font-weight: 600;
  color: rgb(var(--v-theme-on-background));
  margin: 0;
}
/* 浅色极简：大标题 800 字重 */
:global(.v-theme--light) .page-title {
  font-size: var(--text-display);
  font-weight: 800;
  letter-spacing: -1px;
}

.page-subtitle {
  font-size: var(--text-sm);
  color: var(--v-text-muted);
}

.top-bar-right {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.search-field {
  width: 180px;
}

/* 窄屏响应式:主窗口默认 1200,缩放时逐级收窄 */
@media (max-width: 1100px) {
  .search-field { width: 140px; }
}
@media (max-width: 900px) {
  .search-field { width: 120px; }
}
@media (max-width: 720px) {
  /* 极窄屏:隐藏搜索框与显示模式切换,保留视图切换 + 排序 */
  .search-field,
  .top-bar-right > .v-btn-toggle:first-of-type {
    display: none;
  }
}
</style>