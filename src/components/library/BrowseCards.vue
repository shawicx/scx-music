<script setup lang="ts">
import { getGradientForString } from '../../constants/gradients'

export interface CardData {
  name: string
  count: number
}

defineProps<{
  cards: CardData[]
  type: 'albums' | 'artists'
}>()

const emit = defineEmits<{
  'cardClick': [name: string]
}>()
</script>

<template>
  <div class="card-scroll">
    <div
      v-for="card in cards"
      :key="card.name"
      class="browse-card"
      @click="emit('cardClick', card.name)"
    >
      <div
        class="browse-art"
        :class="{ 'artist-art': type === 'artists' }"
        :style="type === 'albums' ? { background: getGradientForString(card.name) } : {}"
      >
        <v-icon :icon="type === 'albums' ? 'mdi-album' : 'mdi-account-music'" size="32" color="white"></v-icon>
      </div>
      <div class="browse-title">{{ card.name }}</div>
      <div class="browse-sub">{{ card.count }} 首</div>
    </div>
  </div>
</template>

<style scoped>
.card-scroll {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 16px;
  align-content: start;
}

.browse-card {
  cursor: pointer;
  transition: transform 0.15s ease;
}

.browse-card:hover {
  transform: translateY(-2px);
}

.browse-art {
  width: 100%;
  aspect-ratio: 1;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 8px;
}

.artist-art {
  background: linear-gradient(135deg, #667eea, #764ba2);
}

.browse-title {
  font-size: var(--text-md);
  font-weight: 500;
  color: rgb(var(--v-theme-on-background));
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.browse-sub {
  font-size: var(--text-xs);
  color: var(--v-text-muted);
}
</style>