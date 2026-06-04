import { ref, computed, type Ref } from 'vue'
import type { SortBy, SortOrder } from '../types'

interface Sortable {
  title: string
  artist: string
  album: string
  durationSecs: number
}

export function useOptimizedSort<T extends Sortable>(items: Ref<T[]>) {
  const sortBy = ref<SortBy>('default')
  const sortOrder = ref<SortOrder>('asc')

  const sorted = computed(() => {
    if (sortBy.value === 'default') return items.value

    const sorted = [...items.value]
    sorted.sort((a, b) => {
      let cmp = 0
      switch (sortBy.value) {
        case 'title':
          cmp = a.title.localeCompare(b.title, 'zh-CN')
          break
        case 'artist':
          cmp = a.artist.localeCompare(b.artist, 'zh-CN')
          break
        case 'album':
          cmp = a.album.localeCompare(b.album, 'zh-CN')
          break
        case 'duration':
          cmp = a.durationSecs - b.durationSecs
          break
      }
      return sortOrder.value === 'asc' ? cmp : -cmp
    })
    return sorted
  })

  return { sortBy, sortOrder, sorted }
}