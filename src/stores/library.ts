import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Song, Playlist, ViewMode, DisplayMode, SortBy, SortOrder } from '../types'
import { getGradientForIndex } from '../constants/gradients'
import { invokeCommand } from '../utils/errorHandler'
import { useToast } from '../composables/useToast'
import { useDebounceSearch } from '../composables/useDebounceSearch'
import { useOptimizedSort } from '../composables/useOptimizedSort'
import i18n from '../i18n'

export const useLibraryStore = defineStore('library', () => {
  // State
  const songs = ref<Song[]>([])
  const currentSongId = ref<string | null>(null)
  const searchQuery = ref('')
  const { debouncedQuery } = useDebounceSearch(searchQuery)
  const viewMode = ref<ViewMode>('list')
  const playlists = ref<Playlist[]>([])
  const playlistSongs = ref<Record<string, string[]>>({})
  const activePlaylistId = ref<string | null>(null)
  const displayMode = ref<DisplayMode>('songs')
  const drilldown = ref<{ type: 'album' | 'artist'; value: string } | null>(null)
  const ready = ref(false)

  const { showSuccess, showError } = useToast()
  const t = i18n.global.t

  // Computed properties
  const currentPlaylistSongs = computed(() => {
    const pid = activePlaylistId.value
    if (!pid) return []
    const ids = playlistSongs.value[pid]
    if (!ids) return []
    const idSet = new Set(ids)
    return songs.value.filter((s) => idSet.has(s.id))
  })

  const searchedSongs = computed(() => {
    let result = currentPlaylistSongs.value
    const q = debouncedQuery.value.toLowerCase()
    if (q) {
      result = result.filter(
        (s) =>
          s.title.toLowerCase().includes(q) ||
          s.artist.toLowerCase().includes(q) ||
          s.album.toLowerCase().includes(q),
      )
    }
    return result
  })

  const drilldownFilter = computed(() => {
    let result = searchedSongs.value
    const d = drilldown.value
    if (d) {
      if (d.type === 'album') {
        result = result.filter((s) => s.album === d.value)
      } else if (d.type === 'artist') {
        result = result.filter((s) => s.artist === d.value)
      }
    }
    return result
  })

  const { sortBy, sortOrder, sorted: displayedSongs } = useOptimizedSort(drilldownFilter)

  
  const filteredAlbums = computed(() => {
    const source = searchedSongs.value
    const map = new Map<string, number>()
    for (const s of source) {
      map.set(s.album, (map.get(s.album) ?? 0) + 1)
    }
    return Array.from(map.entries()).map(([name, count]) => ({ name, count }))
  })

  const filteredArtists = computed(() => {
    const source = searchedSongs.value
    const map = new Map<string, number>()
    for (const s of source) {
      map.set(s.artist, (map.get(s.artist) ?? 0) + 1)
    }
    return Array.from(map.entries()).map(([name, count]) => ({ name, count }))
  })

  const currentSong = computed(() =>
    songs.value.find((s) => s.id === currentSongId.value) ?? null,
  )

  const activePlaylist = computed(() =>
    playlists.value.find((p) => p.id === activePlaylistId.value) ?? null,
  )

  // Actions
  async function loadFromDb() {
    try {
      const data = await invokeCommand<{
        songs: Song[]
        playlists: { id: string; name: string; sort_order: number }[]
        playlistSongs: Record<string, string[]>
        settings: Record<string, string>
      }>('get_bootstrap_data')

      songs.value = data.songs
      playlists.value = data.playlists.map((p) => ({ id: p.id, name: p.name }))
      playlistSongs.value = data.playlistSongs

      currentSongId.value = data.settings['currentSongId'] ?? null
      viewMode.value = (data.settings['viewMode'] as ViewMode) ?? 'list'
      activePlaylistId.value = data.settings['activePlaylistId'] ?? null
      displayMode.value = (data.settings['displayMode'] as DisplayMode) ?? 'songs'

      ready.value = true
    } catch (error) {
      console.error('Failed to load data from database:', error)
      showError(t('toast.loadFailed'))
      throw error
    }
  }

  function saveSetting(key: string, value: string) {
    invokeCommand('set_setting', { key, value }).catch(console.error)
  }

  function setActivePlaylist(id: string) {
    activePlaylistId.value = id
    searchQuery.value = ''
    drilldown.value = null
    saveSetting('activePlaylistId', id)
  }

  function setDisplayMode(mode: DisplayMode) {
    displayMode.value = mode
    drilldown.value = null
    saveSetting('displayMode', mode)
  }

  function setDrilldown(type: 'album' | 'artist', value: string) {
    drilldown.value = { type, value }
  }

  function clearDrilldown() {
    drilldown.value = null
  }

  function setSortBy(value: SortBy) {
    sortBy.value = value
  }

  function setSortOrder(order: SortOrder) {
    sortOrder.value = order
  }

  function toggleSortOrder() {
    sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc'
  }

  async function addPlaylist(name: string) {
    try {
      const pl = await invokeCommand<{ id: string; name: string; sort_order: number }>('create_playlist', { name })
      playlists.value = [...playlists.value, { id: pl.id, name: pl.name }]
      playlistSongs.value = { ...playlistSongs.value, [pl.id]: [] }
      showSuccess(t('toast.playlistCreated', { name }))
      return pl.id
    } catch (error) {
      showError(t('toast.createPlaylistFailed'))
      throw error
    }
  }

  async function renamePlaylist(id: string, name: string) {
    try {
      await invokeCommand('rename_playlist', { id, name })
      playlists.value = playlists.value.map((p) => (p.id === id ? { ...p, name } : p))
      showSuccess(t('toast.playlistRenamed'))
    } catch (error) {
      showError(t('toast.renameFailed'))
      throw error
    }
  }

  async function deletePlaylist(id: string) {
    try {
      await invokeCommand('delete_playlist', { id })
      playlists.value = playlists.value.filter((p) => p.id !== id)
      const updated = { ...playlistSongs.value }
      delete updated[id]
      playlistSongs.value = updated
      if (activePlaylistId.value === id) {
        activePlaylistId.value = playlists.value[0]?.id ?? null
        if (activePlaylistId.value) {
          saveSetting('activePlaylistId', activePlaylistId.value)
        }
      }
      showSuccess(t('toast.playlistDeleted'))
    } catch (error) {
      showError(t('toast.deletePlaylistFailed'))
      throw error
    }
  }

  async function addSongToPlaylist(playlistId: string, songId: string) {
    try {
      const current = playlistSongs.value[playlistId] ?? []
      if (!current.includes(songId)) {
        await invokeCommand('add_songs_to_playlist', { playlistId, songIds: [songId] })
        playlistSongs.value = { ...playlistSongs.value, [playlistId]: [...current, songId] }
        showSuccess(t('toast.addedToPlaylist'))
      }
    } catch (error) {
      showError(t('toast.addToPlaylistFailed'))
      throw error
    }
  }

  async function removeSongFromPlaylist(playlistId: string, songId: string) {
    try {
      await invokeCommand('remove_song_from_playlist', { playlistId, songId })
      const current = playlistSongs.value[playlistId] ?? []
      playlistSongs.value = {
        ...playlistSongs.value,
        [playlistId]: current.filter((id) => id !== songId),
      }
      showSuccess(t('toast.removedFromPlaylist'))
    } catch (error) {
      showError(t('toast.removeFailed'))
      throw error
    }
  }

  async function importToPlaylist(playlistId: string) {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog')
      const selected = await open({
        directory: true,
        multiple: false,
        title: t('sidebar.importFolder'),
      })
      if (!selected) return 0

      const files: Array<{
        id: string
        title: string
        artist: string
        album: string
        duration: string
        duration_secs: number
        quality: string
        file_path: string
      }> = await invokeCommand('scan_music_folder', { dirPath: selected })

      const newSongs: Song[] = files.map((f, i) => ({
        id: f.id,
        title: f.title,
        artist: f.artist,
        album: f.album,
        duration: f.duration,
        durationSecs: f.duration_secs,
        quality: f.quality,
        filePath: f.file_path,
        artGradient: getGradientForIndex(songs.value.length + i),
      }))

      // Upsert to SQLite
      const existingIds = new Set(songs.value.map((s) => s.id))
      const uniqueNew = newSongs.filter((s) => !existingIds.has(s.id))
      if (uniqueNew.length > 0) {
        await invokeCommand('upsert_songs', { songs: uniqueNew })
        songs.value = [...songs.value, ...uniqueNew]
      }

      // Add to playlist in SQLite
      const currentIds = playlistSongs.value[playlistId] ?? []
      const currentSet = new Set(currentIds)
      const newIds = newSongs.filter((s) => !currentSet.has(s.id)).map((s) => s.id)
      if (newIds.length > 0) {
        await invokeCommand('add_songs_to_playlist', { playlistId, songIds: newIds })
        playlistSongs.value = {
          ...playlistSongs.value,
          [playlistId]: [...currentIds, ...newIds],
        }
      }

      showSuccess(t('toast.songsImported', { count: newIds.length }))
      return newIds.length
    } catch (error) {
      showError(t('toast.importFailed'))
      throw error
    }
  }

  function playSong(id: string) {
    currentSongId.value = id
    saveSetting('currentSongId', id)
  }

  return {
    // State
    songs,
    currentSongId,
    searchQuery,
    viewMode,
    playlists,
    playlistSongs,
    activePlaylistId,
    displayMode,
    drilldown,
    sortBy,
    sortOrder,
    ready,

    // Computed
    currentPlaylistSongs,
    searchedSongs,
    displayedSongs,
    filteredAlbums,
    filteredArtists,
    currentSong,
    activePlaylist,

    // Actions
    loadFromDb,
    setActivePlaylist,
    setDisplayMode,
    setDrilldown,
    clearDrilldown,
    setSortBy,
    setSortOrder,
    toggleSortOrder,
    addPlaylist,
    renamePlaylist,
    deletePlaylist,
    addSongToPlaylist,
    removeSongFromPlaylist,
    importToPlaylist,
    playSong,
  }
})