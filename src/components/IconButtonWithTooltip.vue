<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  icon: string
  iconActive?: string
  active?: boolean
  disabled?: boolean
  tooltip: string | (() => string)
  color?: string
  size?: string
}

const props = withDefaults(defineProps<Props>(), {
  active: false,
  disabled: false,
  size: 'small',
})

const emit = defineEmits<{
  click: [e: Event]
}>()

const tooltipText = computed(() =>
  typeof props.tooltip === 'function' ? props.tooltip() : props.tooltip
)
</script>

<template>
  <v-btn
    :icon="true"
    :size="size"
    :variant="'plain'"
    :density="'compact'"
    :disabled="disabled"
    :color="active ? color : undefined"
    @click="emit('click', $event)"
  >
    <v-tooltip activator="parent" location="top">{{ tooltipText }}</v-tooltip>
    <v-icon :icon="active && iconActive ? iconActive : icon"></v-icon>
  </v-btn>
</template>
