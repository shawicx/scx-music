<script setup lang="ts">
defineProps<{ locked: boolean; isDark?: boolean }>()
const emit = defineEmits<{ toggle: [] }>()

function onClick(e: MouseEvent) {
  e.stopPropagation()
  e.preventDefault()
  emit('toggle')
}
function onMousedown(e: MouseEvent) {
  // 阻止冒泡到根元素的 startDragging 处理
  e.stopPropagation()
}
</script>

<template>
  <button
    class="lock-badge"
    :class="{ locked, light: !isDark }"
    :title="locked ? '已锁定' : '锁定（点击穿透）'"
    @click="onClick"
    @mousedown="onMousedown"
  >
    <v-icon :icon="locked ? 'mdi-lock' : 'mdi-lock-open'" size="18" />
  </button>
</template>

<style scoped>
.lock-badge {
  position: absolute;
  top: 8px;
  right: 12px;
  width: 28px;
  height: 28px;
  border-radius: var(--radius-pill);
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 0.2s ease, background 0.2s ease;
}
/* 暗色(默认):黑底白字 */
.lock-badge {
  background: rgba(0, 0, 0, 0.4);
  color: rgba(255, 255, 255, 0.8);
}
.lock-badge:hover {
  background: rgba(0, 0, 0, 0.7);
  opacity: 1;
}
/* 浅色:白底深字 + 细边框 */
.lock-badge.light {
  background: rgba(255, 255, 255, 0.7);
  color: #333;
  border: 1px solid rgba(0, 0, 0, 0.1);
}
.lock-badge.light:hover {
  background: rgba(255, 255, 255, 0.95);
  opacity: 1;
}
.lock-badge.locked {
  opacity: 0.6;
}
</style>
