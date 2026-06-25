<script setup lang="ts">
/**
 * P2 脉冲点：4 个圆点按透明度递增，CSS 呼吸动画，表达「正在播放」。
 * 主题色驱动，不读取音频数据，与可视化零冲突。
 * 用于播放条歌名下方和 NowPlaying 封面下方。
 */
withDefaults(defineProps<{
  /** 圆点直径 px */
  size?: number
  /** 间距 px */
  gap?: number
}>(), {
  size: 4,
  gap: 3,
})
</script>

<template>
  <div class="pulse-dots" :style="{ gap: gap + 'px' }">
    <span
      v-for="i in 4"
      :key="i"
      class="pulse-dot"
      :style="{ width: size + 'px', height: size + 'px', animationDelay: ((i - 1) * 0.15) + 's' }"
    />
  </div>
</template>

<style scoped>
.pulse-dots {
  display: inline-flex;
  align-items: center;
}
.pulse-dot {
  border-radius: 50%;
  background: rgb(var(--v-theme-primary));
  animation: pulse-beat 1.2s ease-in-out infinite;
}
@keyframes pulse-beat {
  0%, 100% { transform: scale(0.7); opacity: 0.3; }
  50% { transform: scale(1.25); opacity: 1; }
}
.pulse-dot:nth-child(1) { opacity: 0.3; }
.pulse-dot:nth-child(2) { opacity: 0.55; }
.pulse-dot:nth-child(3) { opacity: 0.8; }
.pulse-dot:nth-child(4) { opacity: 1; }
</style>
