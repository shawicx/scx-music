import gsap from 'gsap'
import { useAnimation } from './useAnimation'

export function usePageTransition() {
  const { easings } = useAnimation()

  function onEnter(el: Element, done: () => void) {
    gsap.fromTo(el, {
      opacity: 0,
      y: 12,
    }, {
      opacity: 1,
      y: 0,
      duration: 0.28,
      ease: easings.smooth,
      onComplete: done,
    })
  }

  function onLeave(el: Element, done: () => void) {
    gsap.to(el, {
      opacity: 0,
      y: -8,
      duration: 0.2,
      ease: 'power2.in',
      onComplete: done,
    })
  }

  return { onEnter, onLeave }
}
