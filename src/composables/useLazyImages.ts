import { onMounted, onUnmounted } from 'vue'

export function useLazyImages(selector = '[data-lazy-img]') {
  let observer: IntersectionObserver | null = null

  onMounted(() => {
    observer = new IntersectionObserver((entries) => {
      for (const entry of entries) {
        if (entry.isIntersecting) {
          const el = entry.target as HTMLElement
          const src = el.dataset.lazyImg
          if (src) {
            const img = el as HTMLImageElement
            if (img.tagName === 'IMG') {
              img.src = src
            }
            el.removeAttribute('data-lazy-img')
            observer?.unobserve(el)
          }
        }
      }
    })
    document.querySelectorAll(selector).forEach((el) => observer!.observe(el))
  })

  onUnmounted(() => {
    observer?.disconnect()
    observer = null
  })

  function observe(el: HTMLElement) {
    observer?.observe(el)
  }

  return { observe }
}