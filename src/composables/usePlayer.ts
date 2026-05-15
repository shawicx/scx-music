import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { Song } from './useLibrary'

export type PlaybackMode = 'sequential' | 'repeat_all' | 'repeat_one' | 'shuffle'
export type PlaybackState = 'playing' | 'paused' | 'stopped'

const currentSong = ref<Song | null>(null)
const isPlaying = ref(false)
const progress = ref(0)
const duration = ref(0)
const volume = ref(1.0)
const playbackMode = ref<PlaybackMode>('sequential')
const queue = ref<Song[]>([])
const queueIndex = ref(0)

// Global toast state
const toastMsg = ref('')
const toastVisible = ref(false)
let toastTimer: ReturnType<typeof setTimeout> | null = null

export function showToast(msg: string) {
  toastMsg.value = msg
  toastVisible.value = true
  if (toastTimer) clearTimeout(toastTimer)
  toastTimer = setTimeout(() => { toastVisible.value = false }, 2000)
}

let listenersSetup = false

async function setupListeners() {
  if (listenersSetup) return
  listenersSetup = true

  const unlisteners: UnlistenFn[] = []

  unlisteners.push(
    await listen<{ current: number; duration: number }>('audio:progress', (e) => {
      progress.value = e.payload.current
      duration.value = e.payload.duration
    }),
  )

  unlisteners.push(
    await listen<{
      state: PlaybackState
      currentSong: Song | null
      queueIndex: number
      mode: PlaybackMode
    }>('audio:state_change', (e) => {
      isPlaying.value = e.payload.state === 'playing'
      if (e.payload.currentSong) {
        currentSong.value = e.payload.currentSong
      }
      queueIndex.value = e.payload.queueIndex
      playbackMode.value = e.payload.mode
    }),
  )

  unlisteners.push(
    await listen<Song | null>('audio:track_change', (e) => {
      currentSong.value = e.payload
      if (e.payload) {
        duration.value = e.payload.durationSecs
      }
    }),
  )

  unlisteners.push(
    await listen<string>('audio:error', (e) => {
      console.error('Audio error:', e.payload)
    }),
  )
}

function formatTime(secs: number): string {
  if (isNaN(secs) || !isFinite(secs)) return '0:00'
  const s = Math.max(0, Math.floor(secs))
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`
}

const progressFormatted = () => formatTime(progress.value)
const durationFormatted = () => formatTime(duration.value)

export function usePlayer() {
  setupListeners()

  async function playFromQueue(songs: Song[], index: number) {
    queue.value = songs
    queueIndex.value = index
    const mapped = songs.map((s) => ({
      id: s.id,
      title: s.title,
      artist: s.artist,
      album: s.album,
      durationSecs: s.durationSecs,
      quality: s.quality,
      filePath: s.filePath,
    }))
    await invoke('player_set_queue', { songs: mapped, index })
  }

  async function togglePlayPause() {
    if (isPlaying.value) {
      await invoke('player_pause')
    } else {
      await invoke('player_resume')
    }
  }

  async function seek(positionSecs: number) {
    await invoke('player_seek', { positionSecs })
  }

  async function setVolume(v: number) {
    volume.value = v
    await invoke('player_set_volume', { volume: v })
  }

  async function next() {
    if (queue.value.length === 0) return
    if (queueIndex.value >= queue.value.length - 1 && playbackMode.value === 'sequential') {
      showToast('已经是最后一首了')
      return
    }
    await invoke('player_next')
  }

  async function previous() {
    if (queue.value.length === 0) return
    if (queueIndex.value === 0) {
      showToast('已经是第一首了')
      return
    }
    await invoke('player_previous')
  }

  async function setMode(mode: PlaybackMode) {
    playbackMode.value = mode
    await invoke('player_set_mode', { mode })
  }

  async function stop() {
    await invoke('player_stop')
  }

  return {
    currentSong,
    isPlaying,
    progress,
    duration,
    volume,
    playbackMode,
    queue,
    queueIndex,
    toastMsg,
    toastVisible,
    playFromQueue,
    togglePlayPause,
    seek,
    setVolume,
    next,
    previous,
    setMode,
    stop,
    formatTime,
    progressFormatted,
    durationFormatted,
  }
}
