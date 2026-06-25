import gsap from 'gsap'
import { useAnimation } from './useAnimation'

export function usePlayerExpand() {
  const { createTimeline, easings } = useAnimation()

  function onEnter(el: Element, done: () => void) {
    const overlay = el as HTMLElement
    const closeBtn = overlay.querySelector('.close-btn')
    const topSection = overlay.querySelector('.top-section')
    const progressSection = overlay.querySelector('.progress-section')
    const controls = overlay.querySelector('.controls')
    const vignette = overlay.querySelector('.vignette')
    const status = overlay.querySelector('.mode-status-bar')

    const tl = createTimeline({
      onComplete: done,
    })

    // Background fade in
    tl.fromTo(overlay, {
      opacity: 0,
    }, {
      opacity: 1,
      duration: 0.3,
      ease: easings.gentle,
    })

    // Vignette（主题色晕染层）
    if (vignette) {
      tl.fromTo(vignette, {
        opacity: 0,
        scale: 0.8,
      }, {
        opacity: 1,
        scale: 1,
        duration: 0.5,
        ease: easings.smooth,
      }, '<0.1')
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

    // Top section (title + artist)
    if (topSection) {
      tl.fromTo(topSection, {
        opacity: 0,
        y: 16,
      }, {
        opacity: 1,
        y: 0,
        duration: 0.4,
        ease: easings.smooth,
      }, '<0.05')
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
