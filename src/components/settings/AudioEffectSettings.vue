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
            :style="{ height: `${Math.max(2, 8 + g)}px` }"
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
  grid-template-columns: repeat(auto-fill, minmax(190px, 1fr));
  gap: 8px;
  padding: 4px 0 8px;
}

.preset-option {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 8px 12px;
  border-radius: 9px;
  border: var(--hairline-w) solid var(--hairline);
  background: transparent;
  cursor: pointer;
  transition: border-color 0.2s, background 0.2s;
  color: var(--text-1);
  font-size: 12.5px;
  font-family: inherit;
  text-align: left;
}

.preset-option:hover {
  background: var(--surface-2);
}

.preset-option.active {
  border-color: var(--accent);
  background: rgb(var(--accent-rgb) / 0.08);
  font-weight: 560;
}

.preset-name {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 100%;
}

/* 迷你 EQ 曲线：10 频段增益映射为柱高（0dB=8px，±12dB=-4..20px，下限 2px） */
.preset-curve {
  display: flex;
  align-items: center;
  gap: 2px;
  height: 16px;
  flex: 0 0 auto;
}

.preset-curve i {
  width: 3px;
  border-radius: 1.5px;
  background: var(--text-3);
  transition: height 0.2s;
}

.preset-option.active .preset-curve i {
  background: var(--accent);
  opacity: 0.75;
}
</style>
