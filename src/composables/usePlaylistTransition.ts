import gsap from 'gsap'
import { useAnimation } from './useAnimation'

export function usePlaylistTransition() {
  const { easings } = useAnimation()

  function onEnter(el: Element, done: () => void) {
    gsap.fromTo(el, {
      opacity: 0,
      x: 24,
    }, {
      opacity: 1,
      x: 0,
      duration: 0.25,
      ease: easings.smooth,
      onComplete: done,
    })
  }

  function onLeave(el: Element, done: () => void) {
    gsap.to(el, {
      opacity: 0,
      x: -24,
      duration: 0.15,
      ease: 'power2.in',
      onComplete: done,
    })
  }

  return { onEnter, onLeave }
}
