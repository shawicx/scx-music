import gsap from 'gsap'
import { Flip } from 'gsap/Flip'
import { ScrollToPlugin } from 'gsap/ScrollToPlugin'
import { onUnmounted } from 'vue'

gsap.registerPlugin(Flip, ScrollToPlugin)

export const easings = {
  smooth: 'expo.out',
  gentle: 'power2.out',
  bounce: 'back.out(1.4)',
  fluid: 'power3.inOut',
} as const

export function useAnimation() {
  let ctx: gsap.Context | null = null

  function getContext(): gsap.Context {
    if (!ctx) {
      ctx = gsap.context(() => {})
    }
    return ctx
  }

  function createTimeline(vars?: gsap.TimelineVars): gsap.core.Timeline {
    const tl = gsap.timeline(vars)
    getContext().add(() => tl)
    return tl
  }

  function killAll() {
    ctx?.revert()
    ctx = null
  }

  onUnmounted(killAll)

  return { gsap, Flip, createTimeline, killAll, easings }
}
