import { ref, computed, type Ref } from 'vue'

export interface VirtualScrollOptions {
  itemHeight: number // 每个item的高度
  containerHeight: number // 容器高度
  overscan?: number // 预渲染的数量，默认为5
}

export function useVirtualScroll<T>(
  items: Ref<T[]>,
  options: VirtualScrollOptions
) {
  const { itemHeight, containerHeight, overscan = 5 } = options

  const scrollTop = ref(0)
  const containerRef = ref<HTMLElement | null>(null)

  // 计算可见范围
  const visibleRange = computed(() => {
    const start = Math.floor(scrollTop.value / itemHeight)
    const visibleCount = Math.ceil(containerHeight / itemHeight)
    const end = start + visibleCount

    return {
      start: Math.max(0, start - overscan),
      end: Math.min(items.value.length, end + overscan),
    }
  })

  // 可见的数据
  const visibleItems = computed(() => {
    const { start, end } = visibleRange.value
    return items.value.slice(start, end).map((item, index) => ({
      item,
      index: start + index,
    }))
  })

  // 容器样式
  const containerStyle = computed(() => ({
    height: `${containerHeight}px`,
    overflow: 'auto' as const,
    position: 'relative' as const,
  }))

  // 内容区域的样式（总高度）
  const contentStyle = computed(() => ({
    height: `${items.value.length * itemHeight}px`,
    position: 'relative' as const,
  }))

  // 偏移量（让可见内容出现在正确位置）
  const offsetY = computed(() => {
    return visibleRange.value.start * itemHeight
  })

  // 滚动处理
  function handleScroll(event: Event) {
    const target = event.target as HTMLElement
    scrollTop.value = target.scrollTop
  }

  // 滚动到指定索引
  function scrollToIndex(index: number) {
    if (!containerRef.value) return

    const targetScroll = index * itemHeight
    containerRef.value.scrollTop = targetScroll
    scrollTop.value = targetScroll
  }

  return {
    containerRef,
    visibleItems,
    containerStyle,
    contentStyle,
    offsetY,
    handleScroll,
    scrollToIndex,
  }
}