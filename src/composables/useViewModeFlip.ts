import { nextTick, type Ref } from 'vue'
import { Flip } from 'gsap/Flip'
import { useAnimation } from './useAnimation'

export function useViewModeFlip(
  containerRef: Ref<HTMLElement | null>,
  displayMode: Ref<string>,
) {
  const { easings } = useAnimation()
  let isAnimating = false

  async function flipOnChange(applyChange: () => void) {
    if (!containerRef.value || isAnimating) {
      applyChange()
      return
    }

    // Only animate between list and grid when in songs display mode
    if (displayMode.value !== 'songs') {
      applyChange()
      return
    }

    const elements = containerRef.value.querySelectorAll('[data-song-id]')
    if (elements.length === 0) {
      applyChange()
      return
    }

    // 1. Record current positions
    const state = Flip.getState(elements)

    // 2. Apply the DOM change
    applyChange()

    // 3. Wait for Vue to update DOM
    await nextTick()

    // 4. Animate from old positions to new
    isAnimating = true
    Flip.from(state, {
      duration: 0.4,
      ease: easings.fluid,
      stagger: 0.02,
      absolute: true,
      onComplete: () => {
        isAnimating = false
      },
    })
  }

  return { flipOnChange, isAnimating }
}
