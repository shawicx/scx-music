import { watch, type Ref } from 'vue'
import gsap from 'gsap'
import { useAnimation } from './useAnimation'

export function useLyricsAnimation(
  containerRef: Ref<HTMLElement | null>,
  currentLineIndex: Ref<number>,
  userScrolling: Ref<boolean>,
) {
  const { easings, createTimeline } = useAnimation()

  function applySpotlight() {
    const container = containerRef.value
    if (!container) return

    const lineElements = container.querySelectorAll('.lyric-line')
    const currentIdx = currentLineIndex.value

    lineElements.forEach((el, i) => {
      const distance = Math.abs(i - currentIdx)
      let opacity: number
      if (distance === 0) opacity = 1
      else if (distance === 1) opacity = 0.5
      else if (distance === 2) opacity = 0.3
      else opacity = 0.2

      const scale = distance === 0 ? 1.02 : 1

      gsap.to(el, {
        opacity,
        scale,
        duration: 0.3,
        ease: easings.gentle,
      })
    })
  }

  function scrollToCurrent() {
    if (userScrolling.value) return
    const container = containerRef.value
    if (!container) return

    const active = container.querySelector('.lyric-line.active') as HTMLElement
    if (!active) return

    const containerHeight = container.clientHeight
    const targetScroll = active.offsetTop - containerHeight / 2 + active.clientHeight / 2

    gsap.to(container, {
      scrollTo: { y: Math.max(0, targetScroll), autoKill: false },
      duration: 0.35,
      ease: 'power2.inOut',
    })
  }

  function animateSongChange() {
    const container = containerRef.value
    if (!container) return

    const tl = createTimeline()

    const existingLines = container.querySelectorAll('.lyric-line')
    if (existingLines.length > 0) {
      tl.to(existingLines, {
        opacity: 0,
        y: -8,
        duration: 0.2,
        stagger: 0.02,
        ease: 'power2.in',
      })
    }
  }

  // Watch current line index for spotlight + scroll
  watch(currentLineIndex, () => {
    applySpotlight()
    scrollToCurrent()
  })

  return { applySpotlight, scrollToCurrent, animateSongChange }
}
