<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { useAudioEffectsStore } from '../../stores/audioEffects'
import { useI18n } from '../../composables/useI18n'

const { t } = useI18n()
const store = useAudioEffectsStore()
const { presets, currentPresetId, enabled } = storeToRefs(store)
const { switchPreset, setEnabled } = store

// v-switch 的 update:model-value 类型为 boolean | null（indeterminate 态），null 时忽略
function onToggle(v: boolean | null) {
  if (v !== null) setEnabled(v)
}
</script>

<template>
  <v-card class="settings-card" variant="flat" color="surface">
    <div class="card-header">
      <v-icon icon="mdi-tune-variant" size="18" class="card-icon" />
      <span class="card-title">{{ t('settings.eq.title') }}</span>
      <v-switch
        :model-value="enabled"
        density="compact"
        color="primary"
        hide-details
        class="eq-switch"
        :aria-label="t('settings.eq.enabled')"
        @update:model-value="onToggle"
      />
    </div>

    <div class="preset-list">
      <button
        v-for="preset in presets"
        :key="preset.id"
        :class="['preset-option', { active: currentPresetId === preset.id }]"
        @click="switchPreset(preset.id)"
      >
        <span class="preset-name">{{ t(`settings.eq.presets.${preset.id}`) }}</span>
        <span class="preset-curve" aria-hidden="true">
          <i
            v-for="(g, i) in preset.gainsDb"
            :key="i"
            :style="{ height: `${Math.max(2, 12 + g * 2)}px` }"
          />
        </span>
      </button>
    </div>
  </v-card>
</template>

<style src="../../styles/settings-card.css"></style>
<style scoped>
.eq-switch {
  margin-left: auto;
  flex: 0 0 auto;
}

.preset-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 8px;
}

.preset-option {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 8px;
  padding: 10px 14px;
  border-radius: 8px;
  border: 2px solid transparent;
  background: transparent;
  cursor: pointer;
  transition: border-color 0.2s, background 0.2s;
  color: rgb(var(--v-theme-on-background));
  font-size: var(--text-md);
  text-align: left;
}

.preset-option:hover {
  background: var(--v-accent-bg);
}

.preset-option.active {
  border-color: rgb(var(--v-theme-primary));
  background: var(--v-accent-bg);
}

.preset-name {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 100%;
}

/* 迷你 EQ 曲线：10 频段增益映射为柱高（0dB=12px，±12dB=0..36px） */
.preset-curve {
  display: flex;
  align-items: center;
  gap: 3px;
  height: 36px;
}

.preset-curve i {
  width: 4px;
  border-radius: 2px;
  background: rgb(var(--v-theme-primary) / 0.45);
  transition: height 0.2s;
}

.preset-option.active .preset-curve i {
  background: rgb(var(--v-theme-primary));
}
</style>
