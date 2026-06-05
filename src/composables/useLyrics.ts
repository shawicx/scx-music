import { ref, watch, type Ref } from 'vue'
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
}

const LRC_REGEX = /\[(\d{2}):(\d{2})\.(\d{2,3})](.*)/

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
  let unlisten: UnlistenFn | null = null

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
        lines.value = parseLrc(result.rawLrc)
      } else {
        rawLrc.value = null
        source.value = ''
        lines.value = []
      }
    } catch {
      rawLrc.value = null
      lines.value = []
    } finally {
      isLoading.value = false
      currentLineIndex.value = -1
    }
  }

  function computeCurrentLine(progressSecs: number) {
    if (lines.value.length === 0) return
    let idx = -1
    for (let i = 0; i < lines.value.length; i++) {
      if (lines.value[i].time <= progressSecs) {
        idx = i
      } else {
        break
      }
    }
    currentLineIndex.value = idx
  }

  async function setupProgressListener() {
    unlisten = await listen<{ current: number; duration: number }>(
      'audio:progress',
      (e) => {
        computeCurrentLine(e.payload.current)
      },
    )
  }

  watch(currentSong, async (song) => {
    if (song) {
      await fetchLyrics(song)
    } else {
      lines.value = []
      rawLrc.value = null
      currentLineIndex.value = -1
    }
  }, { immediate: true })

  setupProgressListener()

  return {
    lines,
    currentLineIndex,
    isLoading,
    rawLrc,
    source,
  }
}
