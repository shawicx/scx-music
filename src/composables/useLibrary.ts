import { computed, ref } from 'vue'
import { useStorage } from '@vueuse/core'
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

// Global song pool
const songs = useStorage<Song[]>('scx-music-songs', [])
const currentSongId = useStorage<string | null>('scx-music-current-song', null)
const searchQuery = ref('')
const viewMode = useStorage<'list' | 'grid'>('scx-music-view-mode', 'list')

// Playlists
const playlists = useStorage<Playlist[]>('scx-music-playlists', [
  { name: '我喜欢的', id: 'fav' },
  { name: '本地摇滚合集', id: 'rock' },
  { name: '深夜放松', id: 'night' },
  { name: '运动 BGM', id: 'sport' },
])

// Playlist → song IDs mapping
const playlistSongs = useStorage<Record<string, string[]>>('scx-music-playlist-songs', {})

// Currently active playlist
const activePlaylistId = useStorage<string | null>('scx-active-playlist', null)

// Display mode (view form in right panel)
const displayMode = useStorage<'songs' | 'albums' | 'artists'>('scx-display-mode', 'songs')

// Drilldown: when clicking an album/artist card
const drilldown = ref<{ type: 'album' | 'artist'; value: string } | null>(null)

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

// Songs belonging to the active playlist
const currentPlaylistSongs = computed(() => {
  const pid = activePlaylistId.value
  if (!pid) return []
  const ids = playlistSongs.value[pid]
  if (!ids) return []
  const idSet = new Set(ids)
  return songs.value.filter((s) => idSet.has(s.id))
})

// Search applied to current playlist songs
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

// Final displayed songs (after drilldown filter)
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
  return result
})

// Albums derived from searched songs (for card view)
const filteredAlbums = computed(() => {
  const source = searchedSongs.value
  const map = new Map<string, number>()
  for (const s of source) {
    map.set(s.album, (map.get(s.album) ?? 0) + 1)
  }
  return Array.from(map.entries()).map(([name, count]) => ({ name, count }))
})

// Artists derived from searched songs (for card view)
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
  }

  function setDisplayMode(mode: 'songs' | 'albums' | 'artists') {
    displayMode.value = mode
    drilldown.value = null
  }

  function setDrilldown(type: 'album' | 'artist', value: string) {
    drilldown.value = { type, value }
  }

  function clearDrilldown() {
    drilldown.value = null
  }

  function addPlaylist(name: string) {
    const id = 'pl-' + Date.now()
    playlists.value = [...playlists.value, { name, id }]
    return id
  }

  function renamePlaylist(id: string, name: string) {
    playlists.value = playlists.value.map((p) => (p.id === id ? { ...p, name } : p))
  }

  function deletePlaylist(id: string) {
    playlists.value = playlists.value.filter((p) => p.id !== id)
    const updated = { ...playlistSongs.value }
    delete updated[id]
    playlistSongs.value = updated
    if (activePlaylistId.value === id) {
      activePlaylistId.value = playlists.value[0]?.id ?? null
    }
  }

  function addSongToPlaylist(playlistId: string, songId: string) {
    const current = playlistSongs.value[playlistId] ?? []
    if (!current.includes(songId)) {
      playlistSongs.value = { ...playlistSongs.value, [playlistId]: [...current, songId] }
    }
  }

  function removeSongFromPlaylist(playlistId: string, songId: string) {
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

    const newSongs = files.map((f, i) => ({
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

    // Merge into global pool (deduplicate by id)
    const existingIds = new Set(songs.value.map((s) => s.id))
    const uniqueNew = newSongs.filter((s) => !existingIds.has(s.id))
    if (uniqueNew.length > 0) {
      songs.value = [...songs.value, ...uniqueNew]
    }

    // Add IDs to playlist (deduplicate)
    const currentIds = playlistSongs.value[playlistId] ?? []
    const currentSet = new Set(currentIds)
    const newIds = newSongs.filter((s) => !currentSet.has(s.id)).map((s) => s.id)
    if (newIds.length > 0) {
      playlistSongs.value = {
        ...playlistSongs.value,
        [playlistId]: [...currentIds, ...newIds],
      }
    }

    return newIds.length
  }

  function playSong(id: string) {
    currentSongId.value = id
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
    currentSongId,
    currentSong,
    currentPlaylistSongs,
    searchedSongs,
    displayedSongs,
    filteredAlbums,
    filteredArtists,
    setActivePlaylist,
    setDisplayMode,
    setDrilldown,
    clearDrilldown,
    addPlaylist,
    renamePlaylist,
    deletePlaylist,
    addSongToPlaylist,
    removeSongFromPlaylist,
    importToPlaylist,
    playSong,
  }
}
