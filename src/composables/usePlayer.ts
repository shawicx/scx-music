import { ref, computed } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { Song, PlaybackMode, PlaybackState } from '../types'
import { invokeCommand } from '../utils/errorHandler'
import { useToast } from './useToast'
import i18n from '../i18n'
import { generateQueue } from './usePlayQueue'

type PlayerStateReturnType = {
  currentSong: Song | null
  state: PlaybackState
  volume: number
  mode: PlaybackMode
  progress: number
  duration: number
  queueLength: number
  queueIndex: number
}

const currentSong = ref<Song | null>(null)
const isPlaying = ref(false)
const progress = ref(0)
const duration = ref(0)
const volume = ref(1.0)
const playbackMode = ref<PlaybackMode>('sequential')
const queue = ref<Song[]>([])
const queueIndex = ref(0)
const sourceSongs = ref<Song[]>([])
const listenersSetup = ref(false)

const { showToast, showWarning } = useToast()
const t = i18n.global.t

const unlisteners: UnlistenFn[] = []

async function setupListeners() {
  if (listenersSetup.value) return
  listenersSetup.value = true

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

const progressFormatted = computed(() => formatTime(progress.value))
const durationFormatted = computed(() => formatTime(duration.value))

export function usePlayer() {
  async function playFromQueue(songs: Song[], index: number) {
    try {
      sourceSongs.value = songs
      const ordered = generateQueue(songs, index, playbackMode.value)
      queue.value = ordered
      // 找到当前歌曲在 ordered 中的位置
      const playIndex = ordered.findIndex((s) => s.id === songs[index].id)
      queueIndex.value = playIndex >= 0 ? playIndex : 0
      const mapped = ordered.map((s) => ({
        id: s.id,
        title: s.title,
        artist: s.artist,
        album: s.album,
        durationSecs: s.durationSecs,
        quality: s.quality,
        filePath: s.filePath,
      }))
      await invokeCommand('player_set_queue', { songs: mapped, index: queueIndex.value })
    } catch (error) {
      showToast(t('toast.playbackFailed'))
      throw error
    }
  }

  async function regenerateQueue() {
    if (sourceSongs.value.length === 0) return
    const currentSongId = queue.value[queueIndex.value]?.id ?? null
    const sourceIndex = currentSongId
      ? Math.max(0, sourceSongs.value.findIndex((s) => s.id === currentSongId))
      : 0
    const ordered = generateQueue(sourceSongs.value, sourceIndex, playbackMode.value)
    queue.value = ordered
    // 保持当前歌曲在队列中的位置
    let newIndex = 0
    if (currentSongId) {
      const found = ordered.findIndex((s) => s.id === currentSongId)
      if (found >= 0) newIndex = found
    }
    queueIndex.value = newIndex
    const mapped = ordered.map((s) => ({
      id: s.id,
      title: s.title,
      artist: s.artist,
      album: s.album,
      durationSecs: s.durationSecs,
      quality: s.quality,
      filePath: s.filePath,
    }))
    await invokeCommand('player_set_queue', { songs: mapped, index: newIndex })
  }

  async function togglePlayPause() {
    if (!currentSong.value) return
    try {
      if (isPlaying.value) {
        await invokeCommand('player_pause')
      } else {
        await invokeCommand('player_resume')
      }
    } catch (error) {
      showToast(t('toast.playbackControlFailed'))
      throw error
    }
  }

  async function seek(positionSecs: number) {
    try {
      await invokeCommand('player_seek', { positionSecs })
    } catch (error) {
      showToast(t('toast.seekFailed'))
      throw error
    }
  }

  async function setVolume(v: number) {
    try {
      volume.value = v
      await invokeCommand('player_set_volume', { volume: v })
    } catch (error) {
      showToast(t('toast.volumeFailed'))
      throw error
    }
  }

  async function seekRelative(deltaSecs: number) {
    const pos = Math.max(0, Math.min(duration.value, progress.value + deltaSecs))
    await seek(pos)
  }

  async function adjustVolume(delta: number) {
    await setVolume(Math.max(0, Math.min(1, volume.value + delta)))
  }

  async function next() {
    if (queue.value.length === 0) return
    try {
      await invokeCommand('player_next')
    } catch (error) {
      showToast(t('toast.nextSongFailed'))
      throw error
    }
  }

  async function previous() {
    if (queue.value.length === 0) return
    if (queueIndex.value === 0) {
      showWarning(t('toast.firstSong'))
      return
    }
    try {
      await invokeCommand('player_previous')
    } catch (error) {
      showToast(t('toast.previousSongFailed'))
      throw error
    }
  }

  async function setMode(mode: PlaybackMode) {
    const previousMode = playbackMode.value
    try {
      playbackMode.value = mode
      await invokeCommand('player_set_mode', { mode })
    } catch (error) {
      playbackMode.value = previousMode
      showToast(t('toast.modeFailed'))
      throw error
    }
  }

  async function stop() {
    try {
      await invokeCommand('player_stop')
    } catch (error) {
      showToast(t('toast.stopFailed'))
      throw error
    }
  }

  async function getState() {
    try {
      const state = await invokeCommand<PlayerStateReturnType>('player_get_state')
      if (state.currentSong) {
        currentSong.value = state.currentSong
      }
      isPlaying.value = state.state === 'playing'
      progress.value = state.progress
      duration.value = state.duration
      queueIndex.value = state.queueIndex
      playbackMode.value = state.mode
      volume.value = state.volume
    } catch (error) {
      console.log('获取播放状态失败（可能还未开始播放）:', error)
    }
  }

  function updateSongInQueue(updatedSong: Song) {
    queue.value = queue.value.map((s) => (s.id === updatedSong.id ? updatedSong : s))
    if (currentSong.value && currentSong.value.id === updatedSong.id) {
      currentSong.value = updatedSong
    }
  }

  function cleanup() {
    unlisteners.forEach((unlisten) => unlisten())
    unlisteners.length = 0
    listenersSetup.value = false
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
    sourceSongs,
    regenerateQueue,
    listenersSetup,

    setupListeners,
    playFromQueue,
    togglePlayPause,
    seek,
    setVolume,
    seekRelative,
    adjustVolume,
    next,
    previous,
    setMode,
    stop,
    getState,
    updateSongInQueue,
    formatTime,
    cleanup,

    progressFormatted,
    durationFormatted,
  }
}
