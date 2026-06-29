<script setup lang="ts">
import { useDesktopLyrics } from '../composables/useDesktopLyrics'

const { toggleLock, isDark } = useDesktopLyrics()

function onClick() {
  // 锁定窗口只在锁定状态下显示，点击即解锁
  toggleLock(false)
}
</script>

<template>
  <button class="lock-window-btn" :class="{ light: !isDark }" :title="'解锁'" @click="onClick">
    <v-icon icon="mdi-lock" size="20" />
  </button>
</template>

<style src="../styles/window-reset.css"></style>
<style>
html, body, #app {
  background: transparent;
}
</style>

<style scoped>
.lock-window-btn {
  width: 100%;
  height: 100%;
  border: none;
  border-radius: var(--radius-pill);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.2s ease, transform 0.1s ease;
}
/* 暗色(默认):黑底白字 */
.lock-window-btn {
  background: rgba(0, 0, 0, 0.6);
  color: rgba(255, 255, 255, 0.9);
}
.lock-window-btn:hover {
  background: rgba(0, 0, 0, 0.85);
  transform: scale(1.08);
}
/* 浅色:白底深字 + 细边框 */
.lock-window-btn.light {
  background: rgba(255, 255, 255, 0.8);
  color: #333;
  border: 1px solid rgba(0, 0, 0, 0.1);
}
.lock-window-btn.light:hover {
  background: rgba(255, 255, 255, 1);
  transform: scale(1.08);
}
.lock-window-btn:active {
  transform: scale(0.95);
}
</style>
