<script setup lang="ts">
/**
 * 分段控件（曲库浏览方式 / 显示方式 / 时间范围等）。
 * 原型 .seg：surface-2 底 + 选中项 surface-3 浮起。
 */
interface SegOption {
  value: string
  label?: string
  icon?: string
  iconSize?: number
}

withDefaults(
  defineProps<{
    options: SegOption[]
    modelValue: string
    ariaLabel?: string
  }>(),
  { ariaLabel: undefined },
)

const emit = defineEmits<{ 'update:modelValue': [value: string] }>()
</script>

<template>
  <div class="seg" role="group" :aria-label="ariaLabel">
    <button
      v-for="opt in options"
      :key="opt.value"
      type="button"
      class="seg-btn"
      :aria-pressed="modelValue === opt.value"
      @click="emit('update:modelValue', opt.value)"
    >
      <v-icon v-if="opt.icon" :icon="opt.icon" :size="opt.iconSize ?? 14"></v-icon>
      <span v-if="opt.label">{{ opt.label }}</span>
    </button>
  </div>
</template>

<style scoped>
.seg {
  display: flex;
  padding: 2px;
  gap: 2px;
  border-radius: 8px;
  background: var(--surface-2);
  border: var(--hairline-w) solid var(--hairline);
  flex-shrink: 0;
}
.seg-btn {
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 12px;
  color: var(--text-2);
  display: flex;
  align-items: center;
  gap: 5px;
  background: none;
  border: none;
  cursor: pointer;
  font-family: inherit;
  transition: background 0.15s, color 0.15s;
  white-space: nowrap;
}
.seg-btn:hover {
  color: var(--text-1);
}
/* 选中态：暗色轨道上用 10% 白浮起（原 surface-3 与轨道对比过弱），
 * 浅色用纯白浮起，配合字重与投影保证不读内容也能分辨 */
.seg-btn[aria-pressed='true'] {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text-1);
  font-weight: 600;
  box-shadow:
    0 1px 3px rgba(0, 0, 0, 0.3),
    inset 0 1px 0 rgba(255, 255, 255, 0.06);
}
:global(.scx-light) .seg-btn[aria-pressed='true'] {
  background: #ffffff;
  box-shadow: 0 1px 4px rgba(20, 20, 24, 0.14);
}
</style>
