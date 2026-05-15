<script setup lang="ts">
import { computed, ref } from 'vue'
import { useLibrary } from '../composables/useLibrary'
import { usePlayer } from '../composables/usePlayer'

const gradients = [
  'linear-gradient(135deg, #14b8a6, #0d9488)',
  'linear-gradient(135deg, #f093fb, #f5576c)',
  'linear-gradient(135deg, #43e97b, #38f9d7)',
  'linear-gradient(135deg, #fa709a, #fee140)',
  'linear-gradient(135deg, #4facfe, #00f2fe)',
  'linear-gradient(135deg, #a18cd1, #fbc2eb)',
  'linear-gradient(135deg, #667eea, #764ba2)',
  'linear-gradient(135deg, #89f7fe, #66a6ff)',
]

const {
  songs,
  searchQuery,
  viewMode,
  activePlaylist,
  activePlaylistId,
  displayMode,
  drilldown,
  sortBy,
  sortOrder,
  displayedSongs,
  filteredAlbums,
  filteredArtists,
  setDrilldown,
  clearDrilldown,
  setSortBy,
  setSortOrder,
  toggleSortOrder,
  addSongToPlaylist,
  playlists,
} = useLibrary()
const { playFromQueue, currentSong, isPlaying } = usePlayer()

const pageTitle = computed(() => {
  if (!activePlaylist.value) return '选择歌单'
  if (drilldown.value) {
    return drilldown.value.type === 'album' ? drilldown.value.value : drilldown.value.value
  }
  return activePlaylist.value.name
})

const pageSubtitle = computed(() => {
  if (!activePlaylist.value) return ''
  if (drilldown.value) {
    return activePlaylist.value.name
  }
  return ''
})

// Whether to show card browse view (no drilldown, and displayMode is albums or artists)
const showCardView = computed(() => !drilldown.value && (displayMode.value === 'albums' || displayMode.value === 'artists'))

// Cards to show in browse mode
const browseCards = computed(() => {
  if (displayMode.value === 'albums') return filteredAlbums.value
  if (displayMode.value === 'artists') return filteredArtists.value
  return []
})

function onSongClick(index: number) {
  playFromQueue(displayedSongs.value, index)
}

function onCardClick(name: string) {
  if (displayMode.value === 'albums') {
    setDrilldown('album', name)
  } else if (displayMode.value === 'artists') {
    setDrilldown('artist', name)
  }
}

// Add to playlist menu
const songMenu = ref<{ show: boolean; x: number; y: number; songId: string }>({
  show: false, x: 0, y: 0, songId: '',
})

function openSongMenu(e: MouseEvent, songId: string) {
  e.preventDefault()
  songMenu.value = { show: true, x: e.clientX, y: e.clientY, songId }
}

function handleAddToPlaylist(playlistId: string) {
  addSongToPlaylist(playlistId, songMenu.value.songId)
  songMenu.value.show = false
}

// Sort menu
const sortMenu = ref<{ show: boolean; x: number; y: number }>({
  show: false, x: 0, y: 0,
})

const sortOptions = [
  { value: 'default', label: '默认排序', icon: 'mdi-sort-variant' },
  { value: 'title', label: '按标题', icon: 'mdi-format-title' },
  { value: 'artist', label: '按艺术家', icon: 'mdi-account-music' },
  { value: 'album', label: '按专辑', icon: 'mdi-album' },
  { value: 'duration', label: '按时长', icon: 'mdi-clock-outline' },
]

function openSortMenu(e: MouseEvent) {
  e.preventDefault()
  sortMenu.value = { show: true, x: e.clientX, y: e.clientY }
}

function handleSortBy(value: 'title' | 'artist' | 'album' | 'duration' | 'default') {
  if (sortBy.value === value && value !== 'default') {
    toggleSortOrder()
  } else {
    setSortBy(value)
    if (value !== 'default') {
      setSortOrder('asc')
    }
  }
  sortMenu.value.show = false
}

const sortLabel = computed(() => {
  const option = sortOptions.find(o => o.value === sortBy.value)
  if (!option) return '排序'
  if (sortBy.value === 'default') return option.label
  return `${option.label} ${sortOrder.value === 'asc' ? '↑' : '↓'}`
})
</script>

<template>
  <div class="library">
    <div class="top-bar">
      <div class="top-bar-left">
        <v-btn v-if="drilldown" icon size="x-small" variant="plain" density="compact" @click="clearDrilldown">
          <v-icon icon="mdi-chevron-left" size="14"></v-icon>
        </v-btn>
        <div class="title-group">
          <h1 class="page-title">{{ pageTitle }}</h1>
          <span v-if="pageSubtitle" class="page-subtitle">{{ pageSubtitle }}</span>
        </div>
        <v-chip v-if="activePlaylistId && !showCardView" size="x-small" variant="flat" color="surface">
          {{ displayedSongs.length }} 首
        </v-chip>
      </div>
      <div class="top-bar-right">
        <v-text-field
          v-model="searchQuery"
          prepend-inner-icon="mdi-magnify"
          placeholder="搜索歌曲..."
          density="compact"
          variant="solo-filled"
          hide-details
          single-line
          bg-color="surface"
          rounded="lg"
          class="search-field"
        />
        <v-btn-toggle v-model="displayMode" mandatory density="compact" variant="outlined" divided>
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
        <v-btn-toggle v-if="!showCardView && !drilldown" v-model="viewMode" mandatory density="compact" variant="outlined" divided>
          <v-btn value="list" size="small">
            <v-icon icon="mdi-view-list" size="16"></v-icon>
          </v-btn>
          <v-btn value="grid" size="small">
            <v-icon icon="mdi-view-grid" size="16"></v-icon>
          </v-btn>
        </v-btn-toggle>
        <v-btn variant="outlined" append-icon="mdi-sort-variant" @click="openSortMenu">
          {{ sortLabel }}
        </v-btn>
      </div>
    </div>

    <!-- No playlist selected -->
    <div v-if="!activePlaylistId" class="empty-state">
      <v-icon icon="mdi-playlist-music" size="48" color="secondary"></v-icon>
      <p class="empty-text">选择一个歌单开始</p>
      <p class="empty-hint">在左侧点击歌单，或创建新歌单</p>
    </div>

    <!-- Empty playlist -->
    <div v-else-if="!songs.length || displayedSongs.length === 0 && !searchQuery && !showCardView" class="empty-state">
      <v-icon icon="mdi-music" size="48" color="secondary"></v-icon>
      <p class="empty-text">还没有音乐</p>
      <p class="empty-hint">右键点击歌单可以导入音乐</p>
    </div>

    <!-- No search results -->
    <div v-else-if="displayedSongs.length === 0 && searchQuery && !showCardView" class="empty-state">
      <v-icon icon="mdi-magnify" size="48" color="secondary"></v-icon>
      <p class="empty-text">没有找到匹配的歌曲</p>
      <p class="empty-hint">试试其他关键词</p>
    </div>

    <!-- Album/Artist card browse -->
    <div v-else-if="showCardView" class="card-scroll">
      <div
        v-for="card in browseCards"
        :key="card.name"
        class="browse-card"
        @click="onCardClick(card.name)"
      >
        <div
          class="browse-art"
          :class="{ 'artist-art': displayMode === 'artists' }"
          :style="displayMode === 'albums' ? { background: gradients[card.name.length % gradients.length] } : {}"
        >
          <v-icon :icon="displayMode === 'albums' ? 'mdi-album' : 'mdi-account-music'" size="32" color="white"></v-icon>
        </div>
        <div class="browse-title">{{ card.name }}</div>
        <div class="browse-sub">{{ card.count }} 首</div>
      </div>
    </div>

    <!-- List view -->
    <div v-else-if="viewMode === 'list'" class="table-scroll">
      <div class="table-header">
        <div class="col col-num">#</div>
        <div class="col col-title">标题</div>
        <div class="col col-album">专辑</div>
        <div class="col col-artist">艺术家</div>
        <div class="col col-duration">时长</div>
        <div class="col col-actions"></div>
      </div>
      <div class="table-body">
        <div
          v-for="(song, i) in displayedSongs"
          :key="song.id"
          class="table-row"
          :class="{ playing: song.id === currentSong?.id }"
          @click="onSongClick(i)"
          @contextmenu="openSongMenu($event, song.id)"
        >
          <div class="col col-num">
            <v-icon v-if="song.id === currentSong?.id && isPlaying" size="12" icon="mdi-play" color="secondary"></v-icon>
            <v-icon v-else-if="song.id === currentSong?.id" size="12" icon="mdi-pause" color="secondary"></v-icon>
            <span v-else class="row-num">{{ i + 1 }}</span>
          </div>
          <div class="col col-title">
            <div class="song-art" :style="{ background: song.artGradient }" />
            <div class="song-info">
              <div class="song-title" :class="{ active: song.id === currentSong?.id }">{{ song.title }}</div>
              <div class="song-quality">{{ song.quality }}</div>
            </div>
          </div>
          <div class="col col-album">{{ song.album }}</div>
          <div class="col col-artist">{{ song.artist }}</div>
          <div class="col col-duration">{{ song.duration }}</div>
          <div class="col col-actions" @click.stop>
            <v-btn icon size="x-small" variant="plain" density="compact" @click="openSongMenu($event, song.id)">
              <v-icon icon="mdi-dots-horizontal" size="14"></v-icon>
            </v-btn>
          </div>
        </div>
      </div>
    </div>

    <!-- Grid view -->
    <div v-else class="grid-scroll">
      <div
        v-for="(song, i) in displayedSongs"
        :key="song.id"
        class="grid-card"
        :class="{ playing: song.id === currentSong?.id }"
        @click="onSongClick(i)"
        @contextmenu="openSongMenu($event, song.id)"
      >
        <div class="grid-art" :style="{ background: song.artGradient }">
          <v-icon v-if="song.id === currentSong?.id" :icon="isPlaying ? 'mdi-play' : 'mdi-pause'" size="24" color="white" class="grid-play-indicator" />
        </div>
        <div class="grid-title">{{ song.title }}</div>
        <div class="grid-artist">{{ song.artist }}</div>
      </div>
    </div>

    <!-- Song context menu (add to playlist) -->
    <v-menu
      v-model="songMenu.show"
      :target="[songMenu.x, songMenu.y]"
      :close-on-content-click="true"
    >
      <v-list density="compact" min-width="160">
        <v-list-subheader>添加到歌单</v-list-subheader>
        <v-list-item
          v-for="pl in playlists"
          :key="pl.id"
          :title="pl.name"
          @click="handleAddToPlaylist(pl.id)"
        />
      </v-list>
    </v-menu>

    <!-- Sort menu -->
    <v-menu
      v-model="sortMenu.show"
      :target="[sortMenu.x, sortMenu.y]"
      :close-on-content-click="true"
    >
      <v-list density="compact" min-width="160">
        <v-list-subheader>排序方式</v-list-subheader>
        <v-list-item
          v-for="option in sortOptions"
          :key="option.value"
          :prepend-icon="option.icon"
          :title="option.label"
          :active="sortBy === option.value"
          @click="handleSortBy(option.value)"
        >
          <template v-if="sortBy === option.value && option.value !== 'default'" #append>
            <v-icon :icon="sortOrder === 'asc' ? 'mdi-arrow-up' : 'mdi-arrow-down'" size="16" />
          </template>
        </v-list-item>
      </v-list>
    </v-menu>
  </div>
</template>

<style scoped>
.library { flex: 1; display: flex; flex-direction: column; overflow: hidden; }

.top-bar {
  display: flex; justify-content: space-between; align-items: center;
  padding: 12px 20px; border-bottom: 1px solid var(--v-border-color);
}
.top-bar-left { display: flex; align-items: center; gap: 8px; }
.title-group { display: flex; align-items: baseline; gap: 8px; }
.page-title { font-size: var(--text-lg); font-weight: 600; color: rgb(var(--v-theme-on-background)); margin: 0; }
.page-subtitle { font-size: var(--text-sm); color: var(--v-text-muted); }

.top-bar-right { display: flex; align-items: center; gap: 8px; }
.search-field { width: 180px; }

.empty-state { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; opacity: 0.5; }
.empty-text { font-size: var(--text-lg); color: var(--v-text-secondary); }
.empty-hint { font-size: var(--text-sm); color: var(--v-text-muted); }

.table-scroll { flex: 1; overflow-y: auto; }
.table-header {
  display: grid; grid-template-columns: 32px 2.5fr 1.5fr 1.2fr 50px 32px;
  align-items: center; padding: 8px 12px; position: sticky; top: 0;
  background: rgb(var(--v-theme-background)); z-index: 2;
  border-bottom: 1px solid var(--v-border-color);
  font-size: var(--text-xs); color: var(--v-text-muted); text-transform: uppercase; letter-spacing: 0.5px;
}
.table-body { padding: 4px 0; }

.table-row {
  display: grid; grid-template-columns: 32px 2.5fr 1.5fr 1.2fr 50px 32px;
  align-items: center; padding: 6px 12px; border-radius: 8px;
  cursor: pointer; transition: background 0.15s ease;
}
.table-row:hover { background: rgb(var(--v-theme-surface)); }
.table-row.playing { background: rgb(var(--v-theme-surface)); }

.col-num { text-align: center; font-size: var(--text-sm); color: var(--v-text-muted); }
.row-num { color: var(--v-text-muted); }

.col-title { display: flex; align-items: center; gap: 10px; }
.song-art { width: 36px; height: 36px; border-radius: 6px; flex-shrink: 0; }
.song-info { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
.song-title { font-size: var(--text-md); color: rgb(var(--v-theme-on-background)); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.song-title.active { color: rgb(var(--v-theme-secondary)); }
.song-quality { font-size: var(--text-xs); color: var(--v-text-muted); }

.col-album { font-size: var(--text-sm); color: var(--v-text-secondary); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.col-artist { font-size: var(--text-sm); color: var(--v-text-secondary); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.col-duration { font-size: var(--text-sm); color: var(--v-text-muted); text-align: right; }
.col-actions { text-align: center; }

.grid-scroll {
  flex: 1; overflow-y: auto; padding: 16px 20px;
  display: grid; grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
  gap: 16px; align-content: start;
}
.grid-card { cursor: pointer; transition: transform 0.15s ease; }
.grid-card:hover { transform: translateY(-2px); }
.grid-card.playing .grid-art { box-shadow: 0 8px 24px var(--v-accent-shadow); }

.grid-art {
  width: 100%; aspect-ratio: 1; border-radius: 12px;
  display: flex; align-items: center; justify-content: center;
  margin-bottom: 8px; position: relative;
}
.grid-play-indicator { text-shadow: 0 2px 8px rgba(0, 0, 0, 0.3); }
.grid-title { font-size: var(--text-md); font-weight: 500; color: rgb(var(--v-theme-on-background)); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.grid-artist { font-size: var(--text-xs); color: var(--v-text-secondary); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }

.card-scroll {
  flex: 1; overflow-y: auto; padding: 16px 20px;
  display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 16px; align-content: start;
}
.browse-card { cursor: pointer; transition: transform 0.15s ease; }
.browse-card:hover { transform: translateY(-2px); }
.browse-art {
  width: 100%; aspect-ratio: 1; border-radius: 12px;
  display: flex; align-items: center; justify-content: center;
  margin-bottom: 8px;
}
.artist-art { background: linear-gradient(135deg, #667eea, #764ba2); }
.browse-title { font-size: var(--text-md); font-weight: 500; color: rgb(var(--v-theme-on-background)); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.browse-sub { font-size: var(--text-xs); color: var(--v-text-muted); }
</style>
