<script setup lang="ts">
import { ref, computed } from 'vue'

/**
 * 自定义滑杆（进度 / 音量 / 字号等通用）。
 * - 3px 圆角 track，悬停/拖拽时增粗、thumb 浮现
 * - pointer capture 拖拽；键盘 ←/→ 微调（步长可配）
 * - line 变体：2px 细线（播放条/正在播放面板进度）
 * - mini 变体：默认 74px 宽（音量）
 * - fillClass/fillStyle 可覆盖填充色（如深色面板上的白色）
 */
const props = withDefaults(
  defineProps<{
    modelValue: number
    min?: number
    max?: number
    step?: number
    variant?: 'default' | 'line' | 'mini'
    disabled?: boolean
    ariaLabel?: string
    fillStyle?: Record<string, string>
  }>(),
  { min: 0, max: 100, step: 1, variant: 'default', disabled: false, ariaLabel: undefined, fillStyle: undefined },
)

const emit = defineEmits<{
  'update:modelValue': [value: number]
  start: []
  end: []
}>()

const rootEl = ref<HTMLElement | null>(null)
const dragging = ref(false)

const range = computed(() => Math.max(1e-9, props.max - props.min))
const pct = computed(() => {
  const v = Math.min(props.max, Math.max(props.min, props.modelValue))
  return ((v - props.min) / range.value) * 100
})

function commitFromPct(p: number) {
  const clamped = Math.min(1, Math.max(0, p))
  const raw = props.min + clamped * range.value
  // 按 step 对齐，规避浮点尾差
  const stepped = Math.round(raw / props.step) * props.step
  const v = Math.min(props.max, Math.max(props.min, Number(stepped.toFixed(6))))
  if (v !== props.modelValue) {
    emit('update:modelValue', v)
  }
}

function pctFromEvent(e: PointerEvent) {
  const rect = rootEl.value?.getBoundingClientRect()
  if (!rect || rect.width === 0) return 0
  return (e.clientX - rect.left) / rect.width
}

function onPointerDown(e: PointerEvent) {
  if (props.disabled) return
  dragging.value = true
  rootEl.value?.setPointerCapture(e.pointerId)
  emit('start')
  commitFromPct(pctFromEvent(e))
}

function onPointerMove(e: PointerEvent) {
  if (!dragging.value) return
  commitFromPct(pctFromEvent(e))
}

function onPointerUp() {
  if (!dragging.value) return
  dragging.value = false
  emit('end')
}

function onKeydown(e: KeyboardEvent) {
  if (props.disabled) return
  const big = range.value / 10
  let delta = 0
  switch (e.key) {
    case 'ArrowRight':
    case 'ArrowUp':
      delta = e.shiftKey ? big : props.step
      break
    case 'ArrowLeft':
    case 'ArrowDown':
      delta = -(e.shiftKey ? big : props.step)
      break
    case 'Home':
      e.preventDefault()
      emit('update:modelValue', props.min)
      return
    case 'End':
      e.preventDefault()
      emit('update:modelValue', props.max)
      return
    default:
      return
  }
  e.preventDefault()
  const v = Math.min(props.max, Math.max(props.min, props.modelValue + delta))
  emit('update:modelValue', v)
  emit('end')
}
</script>

<template>
  <div
    ref="rootEl"
    class="app-slider"
    :class="[variant, { dragging, disabled }]"
    role="slider"
    :tabindex="disabled ? -1 : 0"
    :aria-label="ariaLabel"
    :aria-valuemin="min"
    :aria-valuemax="max"
    :aria-valuenow="Math.round(modelValue)"
    @pointerdown="onPointerDown"
    @pointermove="onPointerMove"
    @pointerup="onPointerUp"
    @pointercancel="onPointerUp"
    @keydown="onKeydown"
  >
    <div class="track">
      <div class="fill" :style="[{ width: pct + '%' }, fillStyle]"></div>
      <div class="thumb" :style="{ left: pct + '%' }"></div>
    </div>
  </div>
</template>

<style scoped>
.app-slider {
  position: relative;
  height: 16px;
  display: flex;
  align-items: center;
  cursor: pointer;
  touch-action: none;
  min-width: 36px;
}
.app-slider.disabled {
  cursor: default;
  opacity: 0.45;
  pointer-events: none;
}
.app-slider:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
  border-radius: 6px;
}
.track {
  position: relative;
  width: 100%;
  height: 3px;
  border-radius: var(--radius-pill);
  background: var(--hairline-strong);
  overflow: visible;
  transition: height 0.16s;
}
.app-slider:hover .track,
.app-slider.dragging .track {
  height: 5px;
}
.app-slider.line .track {
  height: 2px;
}
.app-slider.line:hover .track,
.app-slider.line.dragging .track {
  height: 3.5px;
}
.fill {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  border-radius: var(--radius-pill);
  background: var(--accent);
}
.thumb {
  position: absolute;
  top: 50%;
  width: 11px;
  height: 11px;
  border-radius: 50%;
  background: #fff;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.4);
  transform: translate(-50%, -50%) scale(0);
  transition: transform 0.16s var(--ease);
  pointer-events: none;
}
.app-slider:hover .thumb,
.app-slider.dragging .thumb,
.app-slider:focus-visible .thumb {
  transform: translate(-50%, -50%) scale(1);
}
.app-slider.mini {
  width: 74px;
}
</style>
