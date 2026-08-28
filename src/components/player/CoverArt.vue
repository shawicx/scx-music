<script setup lang="ts">
import { ref, watch } from 'vue'
import { getCoverUrl } from '../../composables/useCoverArt'

const props = withDefaults(
  defineProps<{
    songId?: string
    fallbackGradient?: string
    iconSize?: number
  }>(),
  {
    songId: undefined,
    fallbackGradient: 'var(--v-gradient-brand)',
    iconSize: 20,
  },
)

const coverUrl = ref<string | null>(null)
const imgShown = ref(false)

watch(
  () => props.songId,
  async (id, oldId) => {
    if (id === oldId) return
    coverUrl.value = null
    imgShown.value = false
    if (!id) return
    const url = await getCoverUrl(id)
    // 等待期间 songId 已再变：丢弃过期结果
    if (props.songId !== id) return
    coverUrl.value = url
  },
  { immediate: true },
)
</script>

<template>
  <div class="cover-art-box" :style="{ background: coverUrl ? 'transparent' : fallbackGradient }">
    <v-icon
      v-if="!coverUrl"
      icon="mdi-music-note"
      :size="iconSize"
      color="rgba(255,255,255,0.6)"
    />
    <img
      v-else
      :src="coverUrl"
      class="cover-img"
      :class="{ shown: imgShown }"
      alt=""
      draggable="false"
      @load="imgShown = true"
    />
  </div>
</template>

<style scoped>
.cover-art-box {
  width: 100%;
  height: 100%;
  border-radius: inherit; /* 圆角跟随父级（PlayerBar 方角 / 全屏页大圆角） */
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  position: relative;
}
.cover-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
  opacity: 0;
  transition: opacity 0.35s ease;
}
.cover-img.shown { opacity: 1; }
</style>
