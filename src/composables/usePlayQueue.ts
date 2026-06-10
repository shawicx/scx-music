import type { Song, PlaybackMode } from '../types'

function fisherYatesShuffle<T>(arr: T[]): T[] {
  const result = [...arr]
  for (let i = result.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1))
    ;[result[i], result[j]] = [result[j], result[i]]
  }
  return result
}

export function generateQueue(
  songs: Song[],
  startIndex: number,
  mode: PlaybackMode,
): Song[] {
  if (songs.length === 0) return []

  const clampIndex = Math.min(startIndex, songs.length - 1)

  switch (mode) {
    case 'sequential':
    case 'repeat_all':
      return [...songs]

    case 'repeat_one':
      return [songs[clampIndex]]

    case 'shuffle': {
      const shuffled = fisherYatesShuffle(songs)
      // 把当前歌曲移到首位
      const currentSong = songs[clampIndex]
      const currentInShuffled = shuffled.findIndex((s) => s.id === currentSong.id)
      if (currentInShuffled > 0) {
        ;[shuffled[0], shuffled[currentInShuffled]] = [shuffled[currentInShuffled], shuffled[0]]
      }
      return shuffled
    }
  }
}
