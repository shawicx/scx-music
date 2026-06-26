import { computed, ref } from 'vue'
import { getGradientForIndex } from '../constants/gradients'
import type { Song, Playlist, ViewMode, DisplayMode, SortBy, SortOrder } from '../types'
import { invokeCommand } from '../utils/errorHandler'
import { useToast } from './useToast'
import { useDebounceSearch } from './useDebounceSearch'
import { useOptimizedSort } from './useOptimizedSort'
import { usePlayerStore } from '../stores/player'
import i18n from '../i18n'

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

export function useLibrary() {
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

  async function clearPlaylist(playlistId: string) {
    try {
      await invokeCommand('clear_playlist', { playlistId })
      playlistSongs.value = { ...playlistSongs.value, [playlistId]: [] }
      showSuccess(t('toast.playlistCleared'))
    } catch (error) {
      showError(t('toast.clearPlaylistFailed'))
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

  async function renameSong(songId: string, newTitle: string, newArtist?: string, newAlbum?: string) {
    try {
      const updated = await invokeCommand<Song>('rename_song', {
        songId,
        newTitle,
        newArtist: newArtist ?? null,
        newAlbum: newAlbum ?? null,
      })
      songs.value = songs.value.map((s) => (s.id === songId ? updated : s))

      const playerStore = usePlayerStore()
      playerStore.updateSongInQueue(updated)

      showSuccess(t('toast.songRenamed'))
    } catch (error) {
      showError(t('toast.songRenameFailed'))
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
        genre: string
        file_size: number
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
        genre: f.genre ?? '',
        fileSize: f.file_size ?? 0,
      }))

      const dbIds: string[] = await invokeCommand('upsert_songs', { songs: newSongs })
      const upsertedSongs = newSongs.map((s, i) => ({ ...s, id: dbIds[i] }))

      const existingIds = new Set(songs.value.map((s) => s.id))
      const addedToState = upsertedSongs.filter((s) => !existingIds.has(s.id))
      if (addedToState.length > 0) {
        songs.value = [...songs.value, ...addedToState]
      }

      const actualIds = upsertedSongs.map((s) => s.id)
      // 原子替换：单事务先删后插，替代 clear_playlist + add_songs_to_playlist 两次串行 IPC
      await invokeCommand('replace_playlist_songs', { playlistId, songIds: actualIds })
      playlistSongs.value = {
        ...playlistSongs.value,
        [playlistId]: actualIds,
      }

      return actualIds.length
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

    currentPlaylistSongs,
    searchedSongs,
    displayedSongs,
    filteredAlbums,
    filteredArtists,
    currentSong,
    activePlaylist,

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
    renameSong,
    clearPlaylist,
    importToPlaylist,
    playSong,
  }
}
