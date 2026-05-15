import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface Song {
  id: string
  title: string
  artist: string
  album: string
  duration: string
  durationSecs: number
  quality: string
  filePath: string
  artGradient: string
}

export interface Playlist {
  name: string
  id: string
}

// Reactive state (no localStorage)
const songs = ref<Song[]>([])
const currentSongId = ref<string | null>(null)
const searchQuery = ref('')
const viewMode = ref<'list' | 'grid'>('list')
const playlists = ref<Playlist[]>([])
const playlistSongs = ref<Record<string, string[]>>({})
const activePlaylistId = ref<string | null>(null)
const displayMode = ref<'songs' | 'albums' | 'artists'>('songs')
const drilldown = ref<{ type: 'album' | 'artist'; value: string } | null>(null)
const sortBy = ref<'title' | 'artist' | 'album' | 'duration' | 'default'>('default')
const sortOrder = ref<'asc' | 'desc'>('asc')
const ready = ref(false)

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

// Load all data from SQLite into reactive state
async function loadFromDb() {
  const [dbSongs, dbPlaylists, settings] = await Promise.all([
    invoke<Song[]>('get_all_songs'),
    invoke<{ id: string; name: string; sort_order: number }[]>('get_playlists'),
    invoke<Record<string, string>>('get_all_settings'),
  ])

  songs.value = dbSongs
  playlists.value = dbPlaylists.map((p) => ({ id: p.id, name: p.name }))

  // Load playlist-song mappings
  const psMap: Record<string, string[]> = {}
  for (const p of dbPlaylists) {
    const pSongs = await invoke<Song[]>('get_playlist_songs', { playlistId: p.id })
    psMap[p.id] = pSongs.map((s) => s.id)
  }
  playlistSongs.value = psMap

  // Restore settings
  currentSongId.value = settings['currentSongId'] ?? null
  viewMode.value = (settings['viewMode'] as 'list' | 'grid') ?? 'list'
  activePlaylistId.value = settings['activePlaylistId'] ?? null
  displayMode.value = (settings['displayMode'] as 'songs' | 'albums' | 'artists') ?? 'songs'

  ready.value = true
}

// Persist a setting to SQLite
function saveSetting(key: string, value: string) {
  invoke('set_setting', { key, value }).catch(console.error)
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
  const q = searchQuery.value.toLowerCase()
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

const displayedSongs = computed(() => {
  let result = searchedSongs.value
  const d = drilldown.value
  if (d) {
    if (d.type === 'album') {
      result = result.filter((s) => s.album === d.value)
    } else if (d.type === 'artist') {
      result = result.filter((s) => s.artist === d.value)
    }
  }

  // Apply sorting
  if (sortBy.value !== 'default') {
    result = [...result].sort((a, b) => {
      let comparison = 0
      switch (sortBy.value) {
        case 'title':
          comparison = a.title.localeCompare(b.title, 'zh-CN')
          break
        case 'artist':
          comparison = a.artist.localeCompare(b.artist, 'zh-CN')
          break
        case 'album':
          comparison = a.album.localeCompare(b.album, 'zh-CN')
          break
        case 'duration':
          comparison = a.durationSecs - b.durationSecs
          break
      }
      return sortOrder.value === 'asc' ? comparison : -comparison
    })
  }

  return result
})

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

  function setDisplayMode(mode: 'songs' | 'albums' | 'artists') {
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

  function setSortBy(value: 'title' | 'artist' | 'album' | 'duration' | 'default') {
    sortBy.value = value
  }

  function setSortOrder(order: 'asc' | 'desc') {
    sortOrder.value = order
  }

  function toggleSortOrder() {
    sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc'
  }

  async function addPlaylist(name: string) {
    const pl = await invoke<{ id: string; name: string; sort_order: number }>('create_playlist', { name })
    playlists.value = [...playlists.value, { id: pl.id, name: pl.name }]
    playlistSongs.value = { ...playlistSongs.value, [pl.id]: [] }
    return pl.id
  }

  async function renamePlaylist(id: string, name: string) {
    await invoke('rename_playlist', { id, name })
    playlists.value = playlists.value.map((p) => (p.id === id ? { ...p, name } : p))
  }

  async function deletePlaylist(id: string) {
    await invoke('delete_playlist', { id })
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
  }

  async function addSongToPlaylist(playlistId: string, songId: string) {
    const current = playlistSongs.value[playlistId] ?? []
    if (!current.includes(songId)) {
      await invoke('add_songs_to_playlist', { playlistId, songIds: [songId] })
      playlistSongs.value = { ...playlistSongs.value, [playlistId]: [...current, songId] }
    }
  }

  async function removeSongFromPlaylist(playlistId: string, songId: string) {
    await invoke('remove_song_from_playlist', { playlistId, songId })
    const current = playlistSongs.value[playlistId] ?? []
    playlistSongs.value = {
      ...playlistSongs.value,
      [playlistId]: current.filter((id) => id !== songId),
    }
  }

  async function importToPlaylist(playlistId: string) {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择音乐文件夹',
    })
    if (!selected) return

    const files: Array<{
      id: string
      title: string
      artist: string
      album: string
      duration: string
      duration_secs: number
      quality: string
      file_path: string
    }> = await invoke('scan_music_folder', { dirPath: selected })

    const newSongs: Song[] = files.map((f, i) => ({
      id: f.id,
      title: f.title,
      artist: f.artist,
      album: f.album,
      duration: f.duration,
      durationSecs: f.duration_secs,
      quality: f.quality,
      filePath: f.file_path,
      artGradient: gradients[(songs.value.length + i) % gradients.length],
    }))

    // Upsert to SQLite
    const existingIds = new Set(songs.value.map((s) => s.id))
    const uniqueNew = newSongs.filter((s) => !existingIds.has(s.id))
    if (uniqueNew.length > 0) {
      await invoke('upsert_songs', { songs: uniqueNew })
      songs.value = [...songs.value, ...uniqueNew]
    }

    // Add to playlist in SQLite
    const currentIds = playlistSongs.value[playlistId] ?? []
    const currentSet = new Set(currentIds)
    const newIds = newSongs.filter((s) => !currentSet.has(s.id)).map((s) => s.id)
    if (newIds.length > 0) {
      await invoke('add_songs_to_playlist', { playlistId, songIds: newIds })
      playlistSongs.value = {
        ...playlistSongs.value,
        [playlistId]: [...currentIds, ...newIds],
      }
    }

    return newIds.length
  }

  function playSong(id: string) {
    currentSongId.value = id
    saveSetting('currentSongId', id)
  }

  return {
    songs,
    searchQuery,
    viewMode,
    playlists,
    playlistSongs,
    activePlaylistId,
    activePlaylist,
    displayMode,
    drilldown,
    sortBy,
    sortOrder,
    currentSongId,
    currentSong,
    currentPlaylistSongs,
    searchedSongs,
    displayedSongs,
    filteredAlbums,
    filteredArtists,
    ready,
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
    loadFromDb,
  }
}
