import { ref, watch, type Ref } from 'vue'
import { useDebounceFn } from '@vueuse/core'

export function useDebounceSearch(source: Ref<string>, delay = 300) {
  const debouncedQuery = ref(source.value)

  const update = useDebounceFn((val: string) => {
    debouncedQuery.value = val
  }, delay)

  watch(source, (val) => update(val))

  return { debouncedQuery }
}