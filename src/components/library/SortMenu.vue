<script setup lang="ts">
import type { SortBy } from '../../types'

const props = defineProps<{
  show: boolean
  x: number
  y: number
  sortBy: SortBy
  sortOrder: 'asc' | 'desc'
}>()

const emit = defineEmits<{
  'update:show': [value: boolean]
  'sort': [value: SortBy]
}>()

const sortOptions = [
  { value: 'default' as const, label: '默认排序', icon: 'mdi-sort-variant' },
  { value: 'title' as const, label: '按标题', icon: 'mdi-format-title' },
  { value: 'artist' as const, label: '按艺术家', icon: 'mdi-account-music' },
  { value: 'album' as const, label: '按专辑', icon: 'mdi-album' },
  { value: 'duration' as const, label: '按时长', icon: 'mdi-clock-outline' },
]

function handleSortBy(value: SortBy) {
  emit('sort', value)
  emit('update:show', false)
}
</script>

<template>
  <v-menu
    :model-value="show"
    @update:model-value="emit('update:show', $event)"
    :target="[x, y]"
    :close-on-content-click="true"
  >
    <v-list density="compact" min-width="160">
      <v-list-subheader>排序方式</v-list-subheader>
      <v-list-item
        v-for="option in sortOptions"
        :key="option.value"
        :prepend-icon="option.icon"
        :title="option.label"
        :active="props.sortBy === option.value"
        @click="handleSortBy(option.value)"
      >
        <template v-if="props.sortBy === option.value && option.value !== 'default'" #append>
          <v-icon :icon="props.sortOrder === 'asc' ? 'mdi-arrow-up' : 'mdi-arrow-down'" size="16" />
        </template>
      </v-list-item>
    </v-list>
  </v-menu>
</template>