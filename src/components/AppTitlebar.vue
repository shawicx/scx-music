<script setup lang="ts">
import { onMounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'

// macOS overlay 标题栏：整条可拖拽（仅在系统真正吃进 titleBarStyle: Overlay 时挂载，
// 见 App.vue 的 get_titlebar_inset_height 探测）。
// data-tauri-drag-region 只在 mousedown 命中携带该属性的元素本身时生效，
// 被子元素挡住的区域全部失效，因此改用显式 startDragging。
// 不绘制标题文字：应用名由侧栏 brand 呈现，避免与系统菜单栏/原生标题重复。

const INTERACTIVE = 'button, a, input, select, textarea, [role="button"]'

function onBarMouseDown(e: MouseEvent) {
  if (e.button !== 0) return
  const target = e.target as HTMLElement | null
  if (target?.closest?.(INTERACTIVE)) return
  try {
    void getCurrentWindow().startDragging()
  } catch {
    // 纯浏览器 dev 环境（无 Tauri IPC）下忽略
  }
}

onMounted(() => {
  // 清掉原生标题文字（overlay 模式下 macOS 会在红绿灯旁绘制）。
  // document.title 必须一并清空：wry 会把 webview 标题同步回窗口标题，
  // 只调 setTitle('') 会被 index.html 的 <title>scx-music</title> 复活。
  try {
    void getCurrentWindow().setTitle('')
  } catch {
    // 同上：浏览器 dev 环境忽略
  }
  document.title = ''
})
</script>

<template>
  <div class="titlebar" @mousedown="onBarMouseDown">
    <div class="traffic" aria-hidden="true"><i></i><i></i><i></i></div>
  </div>
</template>

<style scoped>
.titlebar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 14px 0 20px;
  position: relative;
  z-index: 40;
  background: transparent;
  flex-shrink: 0;
  cursor: default;
  user-select: none;
  -webkit-user-select: none;
}
.traffic {
  display: flex;
  gap: 8px;
  padding: 4px 2px;
  /* 红绿灯占位：仅留出点击区域，按钮由系统绘制在此区域上方 */
  width: 58px;
}
.traffic i {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  display: block;
  opacity: 0;
}
</style>
