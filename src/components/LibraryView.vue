<script setup lang="ts">
import { computed, ref } from 'vue'
import { useLibrary } from '../composables/useLibrary'
import { usePlayer } from '../composables/usePlayer'
import LibraryHeader from './library/LibraryHeader.vue'
import SongTable from './library/SongTable.vue'
import SongGrid from './library/SongGrid.vue'
import BrowseCards from './library/BrowseCards.vue'
import EmptyStates from './library/EmptyStates.vue'
import SortMenu from './library/SortMenu.vue'

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

// Empty state type
const emptyStateType = computed(() => {
  if (!activePlaylistId.value) return 'noPlaylist'
  if (!songs.value.length || (displayedSongs.value.length === 0 && !searchQuery.value && !showCardView.value)) return 'emptyPlaylist'
  if (displayedSongs.value.length === 0 && searchQuery.value && !showCardView.value) return 'noResults'
  return null
})
</script>

<template>
  <div class="library">
    <!-- Header -->
    <LibraryHeader
      :page-title="pageTitle"
      :page-subtitle="pageSubtitle"
      :song-count="displayedSongs.length"
      :search-query="searchQuery"
      :display-mode="displayMode"
      :view-mode="viewMode"
      :sort-by="sortBy"
      :sort-order="sortOrder"
      :show-sort-option="!showCardView && !drilldown"
      @update:search-query="searchQuery = $event"
      @update:display-mode="displayMode = $event"
      @update:view-mode="viewMode = $event"
      @open-sort-menu="openSortMenu"
      @back="clearDrilldown"
    />

    <!-- Empty states -->
    <EmptyStates v-if="emptyStateType" :type="emptyStateType" />

    <!-- Album/Artist card browse -->
    <BrowseCards
      v-else-if="showCardView"
      :cards="browseCards"
      :type="displayMode as 'albums' | 'artists'"
      @card-click="onCardClick"
    />

    <!-- List view -->
    <SongTable
      v-else-if="viewMode === 'list'"
      :songs="displayedSongs"
      :current-song-id="currentSong?.id"
      :is-playing="isPlaying"
      @song-click="onSongClick"
      @song-menu="openSongMenu"
    />

    <!-- Grid view -->
    <SongGrid
      v-else
      :songs="displayedSongs"
      :current-song-id="currentSong?.id"
      :is-playing="isPlaying"
      @song-click="onSongClick"
      @song-menu="openSongMenu"
    />

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
    <SortMenu
      :show="sortMenu.show"
      :x="sortMenu.x"
      :y="sortMenu.y"
      :sort-by="sortBy"
      :sort-order="sortOrder"
      @update:show="sortMenu.show = $event"
      @sort="handleSortBy"
    />
  </div>
</template>

<style scoped>
.library {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
</style>