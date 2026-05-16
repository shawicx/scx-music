import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { Song, PlaybackMode, PlaybackState } from '../types'
import { invokeCommand } from '../utils/errorHandler'
import { useToast } from '../composables/useToast'

export const usePlayerStore = defineStore('player', () => {
  // State
  const currentSong = ref<Song | null>(null)
  const isPlaying = ref(false)
  const progress = ref(0)
  const duration = ref(0)
  const volume = ref(1.0)
  const playbackMode = ref<PlaybackMode>('sequential')
  const queue = ref<Song[]>([])
  const queueIndex = ref(0)
  const listenersSetup = ref(false)

  // Toast
  const { showToast, showWarning } = useToast()

  // Listeners management
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

  // Actions
  async function playFromQueue(songs: Song[], index: number) {
    try {
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
      await invokeCommand('player_set_queue', { songs: mapped, index })
    } catch (error) {
      showToast('播放失败，请重试')
      throw error
    }
  }

  async function togglePlayPause() {
    try {
      if (isPlaying.value) {
        await invokeCommand('player_pause')
      } else {
        await invokeCommand('player_resume')
      }
    } catch (error) {
      showToast('播放控制失败')
      throw error
    }
  }

  async function seek(positionSecs: number) {
    try {
      await invokeCommand('player_seek', { positionSecs })
    } catch (error) {
      showToast('进度调整失败')
      throw error
    }
  }

  async function setVolume(v: number) {
    try {
      volume.value = v
      await invokeCommand('player_set_volume', { volume: v })
    } catch (error) {
      showToast('音量设置失败')
      throw error
    }
  }

  async function next() {
    if (queue.value.length === 0) return
    if (queueIndex.value >= queue.value.length - 1 && playbackMode.value === 'sequential') {
      showWarning('已经是最后一首了')
      return
    }
    try {
      await invokeCommand('player_next')
    } catch (error) {
      showToast('切换下一首失败')
      throw error
    }
  }

  async function previous() {
    if (queue.value.length === 0) return
    if (queueIndex.value === 0) {
      showWarning('已经是第一首了')
      return
    }
    try {
      await invokeCommand('player_previous')
    } catch (error) {
      showToast('切换上一首失败')
      throw error
    }
  }

  async function setMode(mode: PlaybackMode) {
    try {
      playbackMode.value = mode
      await invokeCommand('player_set_mode', { mode })
    } catch (error) {
      showToast('播放模式设置失败')
      throw error
    }
  }

  async function stop() {
    try {
      await invokeCommand('player_stop')
    } catch (error) {
      showToast('停止播放失败')
      throw error
    }
  }

  // Utility functions
  function formatTime(secs: number): string {
    if (isNaN(secs) || !isFinite(secs)) return '0:00'
    const s = Math.max(0, Math.floor(secs))
    return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`
  }

  const progressFormatted = computed(() => formatTime(progress.value))
  const durationFormatted = computed(() => formatTime(duration.value))

  // Cleanup
  function cleanup() {
    unlisteners.forEach(unlisten => unlisten())
    unlisteners.length = 0
    listenersSetup.value = false
  }

  return {
    // State
    currentSong,
    isPlaying,
    progress,
    duration,
    volume,
    playbackMode,
    queue,
    queueIndex,
    listenersSetup,

    // Actions
    setupListeners,
    playFromQueue,
    togglePlayPause,
    seek,
    setVolume,
    next,
    previous,
    setMode,
    stop,
    formatTime,
    cleanup,

    // Computed
    progressFormatted,
    durationFormatted,
  }
})