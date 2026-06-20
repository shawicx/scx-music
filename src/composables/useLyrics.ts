import { ref, watch, onUnmounted, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { Song } from '../types'

export interface LrcLine {
  time: number
  text: string
}

interface LyricsResult {
  rawLrc: string | null
  source: string
  offsetSecs: number
}

const LRC_REGEX = /\[(\d{2}):(\d{2})\.(\d{2,3})](.*)/
const OFFSET_MIN = -10.0
const OFFSET_MAX = 10.0

function parseLrc(raw: string): LrcLine[] {
  const lines: LrcLine[] = []
  for (const line of raw.split('\n')) {
    const match = line.trim().match(LRC_REGEX)
    if (match) {
      const min = parseInt(match[1])
      const sec = parseInt(match[2])
      const ms = match[3].length === 2 ? parseInt(match[3]) * 10 : parseInt(match[3])
      lines.push({
        time: min * 60 + sec + ms / 1000,
        text: match[4].trim(),
      })
    }
  }
  lines.sort((a, b) => a.time - b.time)
  return lines
}

export function useLyrics(currentSong: Ref<Song | null>) {
  const lines = ref<LrcLine[]>([])
  const currentLineIndex = ref(-1)
  const isLoading = ref(false)
  const rawLrc = ref<string | null>(null)
  const source = ref('')
  const offsetSecs = ref(0)
  let _unlisten: UnlistenFn | null = null
  let _listenPromise: Promise<void> | null = null

  async function fetchLyrics(song: Song) {
    isLoading.value = true
    try {
      const result = await invoke<LyricsResult | null>('get_lyrics', {
        songId: song.id,
        filePath: song.filePath,
        title: song.title,
        artist: song.artist,
        durationSecs: song.durationSecs,
      })
      if (result && result.rawLrc) {
        rawLrc.value = result.rawLrc
        source.value = result.source
        offsetSecs.value = result.offsetSecs
        lines.value = parseLrc(result.rawLrc)
      } else {
        rawLrc.value = null
        source.value = ''
        offsetSecs.value = result?.offsetSecs ?? 0
        lines.value = []
      }
    } catch {
      rawLrc.value = null
      lines.value = []
      offsetSecs.value = 0
    } finally {
      isLoading.value = false
      currentLineIndex.value = -1
    }
  }

  function computeCurrentLine(progressSecs: number) {
    if (lines.value.length === 0) return
    const adjusted = progressSecs + offsetSecs.value
    let idx = -1
    for (let i = 0; i < lines.value.length; i++) {
      if (lines.value[i].time <= adjusted) {
        idx = i
      } else {
        break
      }
    }
    currentLineIndex.value = idx
  }

  async function adjustOffset(delta: number) {
    const next = Math.round((offsetSecs.value + delta) * 10) / 10
    if (next < OFFSET_MIN || next > OFFSET_MAX) return
    offsetSecs.value = next
    const song = currentSong.value
    if (song) {
      try {
        await invoke('set_lyric_offset', { songId: song.id, offsetSecs: next })
      } catch {
        // silently fail — offset still works in memory
      }
    }
  }

  async function resetOffset() {
    offsetSecs.value = 0
    const song = currentSong.value
    if (song) {
      try {
        await invoke('set_lyric_offset', { songId: song.id, offsetSecs: 0 })
      } catch {
        // silently fail
      }
    }
  }

  function getSeekTime(lineTime: number): number {
    return lineTime - offsetSecs.value
  }

  function setupProgressListener() {
    if (_listenPromise) return // 已在注册中
    _listenPromise = listen<{ current: number; duration: number }>(
      'audio:progress',
      (e) => {
        computeCurrentLine(e.payload.current)
      },
    )
      .then((un) => {
        _unlisten = un
      })
      .catch((e) => {
        console.error('[useLyrics] listen audio:progress failed:', e)
      })
  }

  watch(currentSong, async (song, oldSong) => {
    if (song && song.id !== oldSong?.id) {
      await fetchLyrics(song)
    } else if (!song) {
      lines.value = []
      rawLrc.value = null
      currentLineIndex.value = -1
      offsetSecs.value = 0
    }
  }, { immediate: true })

  setupProgressListener()

  onUnmounted(async () => {
    // 等待 listen promise 完成后再 unlisten，避免 cleanup 早于 listen 完成导致泄漏
    if (_listenPromise) {
      await _listenPromise.catch(() => {})
    }
    _unlisten?.()
    _unlisten = null
    _listenPromise = null
  })

  return {
    lines,
    currentLineIndex,
    isLoading,
    rawLrc,
    source,
    offsetSecs,
    adjustOffset,
    resetOffset,
    getSeekTime,
  }
}
