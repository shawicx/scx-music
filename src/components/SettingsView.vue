<script setup lang="ts">
import { useTheme } from '../composables/useTheme'
import { themeMeta, type ThemeName } from '../plugins/vuetify'

const emit = defineEmits<{ back: [] }>()
const { themeName, setTheme } = useTheme()

const themes = Object.entries(themeMeta) as [ThemeName, { label: string; color: string }][]
</script>

<template>
  <div class="settings-view">
    <div class="settings-header">
      <v-btn icon variant="plain" size="small" @click="emit('back')">
        <v-icon icon="mdi-arrow-left" size="20" />
      </v-btn>
      <h2 class="settings-title">设置</h2>
    </div>

    <v-card class="settings-card" variant="flat" color="surface">
      <div class="card-header">
        <v-icon icon="mdi-palette" size="18" class="card-icon" />
        <span class="card-title">主题颜色</span>
      </div>

      <div class="theme-grid">
        <button
          v-for="[key, meta] in themes"
          :key="key"
          :class="['theme-option', { active: themeName === key }]"
          @click="setTheme(key)"
        >
          <span class="theme-swatch" :style="{ background: meta.color }" />
          <span class="theme-label">{{ meta.label }}</span>
        </button>
      </div>
    </v-card>
  </div>
</template>

<style scoped>
.settings-view {
  padding: 32px;
  overflow-y: auto;
  height: 100%;
}

.settings-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 24px;
}

.settings-title {
  font-size: var(--text-xl);
  font-weight: 700;
  color: rgb(var(--v-theme-on-background));
}

.settings-card {
  padding: 20px;
  margin-bottom: 16px;
  border-radius: 12px;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 16px;
}

.card-icon {
  color: rgb(var(--v-theme-primary));
}

.card-title {
  font-size: var(--text-md);
  font-weight: 600;
}

.theme-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
}

.theme-option {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 14px 8px;
  border-radius: 12px;
  border: 2px solid transparent;
  background: transparent;
  cursor: pointer;
  transition: border-color 0.2s, background 0.2s;
}

.theme-option:hover {
  background: var(--v-accent-bg);
}

.theme-option.active {
  border-color: rgb(var(--v-theme-primary));
  background: var(--v-accent-bg);
}

.theme-swatch {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  display: block;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

.theme-label {
  font-size: var(--text-sm);
  color: var(--v-text-secondary);
}
</style>
