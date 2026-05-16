<script setup lang="ts">
defineProps<{
  type: 'noPlaylist' | 'emptyPlaylist' | 'noResults'
}>()

const stateConfig = {
  noPlaylist: {
    icon: 'mdi-playlist-music',
    title: '选择一个歌单开始',
    hint: '在左侧点击歌单，或创建新歌单',
  },
  emptyPlaylist: {
    icon: 'mdi-music',
    title: '还没有音乐',
    hint: '右键点击歌单可以导入音乐',
  },
  noResults: {
    icon: 'mdi-magnify',
    title: '没有找到匹配的歌曲',
    hint: '试试其他关键词',
  },
}
</script>

<template>
  <div class="empty-state">
    <v-icon :icon="stateConfig[type].icon" size="48" color="secondary"></v-icon>
    <p class="empty-text">{{ stateConfig[type].title }}</p>
    <p class="empty-hint">{{ stateConfig[type].hint }}</p>
  </div>
</template>

<style scoped>
.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  opacity: 0.5;
}

.empty-text {
  font-size: var(--text-lg);
  color: var(--v-text-secondary);
}

.empty-hint {
  font-size: var(--text-sm);
  color: var(--v-text-muted);
}
</style>