import gsap from 'gsap'
import { useAnimation } from './useAnimation'

export function usePlayerExpand() {
  const { createTimeline, easings } = useAnimation()

  function onEnter(el: Element, done: () => void) {
    const overlay = el as HTMLElement
    const bgLayer = overlay.querySelector('.bg-layer')
    const closeBtn = overlay.querySelector('.close-btn')
    const status = overlay.querySelector('.mode-status-bar')
    const coverColumn = overlay.querySelector('.cover-column')
    const lyricsColumn = overlay.querySelector('.lyrics-column')
    const progressSection = overlay.querySelector('.progress-section')
    const controls = overlay.querySelector('.controls')

    const tl = createTimeline({
      onComplete: done,
    })

    // 背景色整体渐显（无封面回退时依然有意义）
    tl.fromTo(overlay, {
      opacity: 0,
    }, {
      opacity: 1,
      duration: 0.3,
      ease: easings.gentle,
    })

    // 背景层（含模糊封面 + vignette，2026-08-27 重设计后并入同一层）
    if (bgLayer) {
      tl.fromTo(bgLayer, {
        opacity: 0,
      }, {
        opacity: 1,
        duration: 0.4,
        ease: easings.smooth,
      }, '<0.05')
    }

    // Close button
    if (closeBtn) {
      tl.fromTo(closeBtn, {
        opacity: 0,
        y: 12,
      }, {
        opacity: 1,
        y: 0,
        duration: 0.3,
        ease: easings.smooth,
      }, '<0.05')
    }

    // Status bar
    if (status) {
      tl.fromTo(status, {
        opacity: 0,
        y: 12,
      }, {
        opacity: 1,
        y: 0,
        duration: 0.3,
        ease: easings.smooth,
      }, '<')
    }

    // 封面列：轻微生长入场
    if (coverColumn) {
      tl.fromTo(coverColumn, {
        opacity: 0,
        scale: 0.94,
      }, {
        opacity: 1,
        scale: 1,
        duration: 0.45,
        ease: easings.smooth,
      }, '<0.05')
    }

    // 歌词列（歌名/艺术家 + 歌词）
    if (lyricsColumn) {
      tl.fromTo(lyricsColumn, {
        opacity: 0,
        y: 16,
      }, {
        opacity: 1,
        y: 0,
        duration: 0.4,
        ease: easings.smooth,
      }, '<0.1')
    }

    // Progress section
    if (progressSection) {
      tl.fromTo(progressSection, {
        opacity: 0,
        y: 16,
      }, {
        opacity: 1,
        y: 0,
        duration: 0.3,
        ease: easings.smooth,
      }, '<0.1')
    }

    // Controls
    if (controls) {
      tl.fromTo(controls, {
        opacity: 0,
        y: 16,
      }, {
        opacity: 1,
        y: 0,
        duration: 0.35,
        ease: easings.smooth,
      }, '<0.05')
    }
  }

  function onLeave(el: Element, done: () => void) {
    const overlay = el as HTMLElement
    gsap.to(overlay, {
      opacity: 0,
      scale: 0.98,
      duration: 0.25,
      ease: 'power2.in',
      onComplete: done,
    })
  }

  return { onEnter, onLeave }
}
