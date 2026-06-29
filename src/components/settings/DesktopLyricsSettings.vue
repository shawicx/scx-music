<script setup lang="ts">
import { useDesktopLyrics, type GlowStrength } from '../../composables/useDesktopLyrics'
import { useI18n } from '../../composables/useI18n'

const { t } = useI18n()
const {
  config: desktopLyricsConfig,
  locked: desktopLyricsLocked,
  toggle: toggleDesktopLyrics,
  toggleLock: toggleDesktopLyricsLock,
  updateConfig: updateDesktopLyricsConfig,
} = useDesktopLyrics()

const glowOptions: { value: GlowStrength; labelKey: string }[] = [
  { value: 'off', labelKey: 'lyrics.desktopLyrics.glowOff' },
  { value: 'weak', labelKey: 'lyrics.desktopLyrics.glowWeak' },
  { value: 'medium', labelKey: 'lyrics.desktopLyrics.glowMedium' },
  { value: 'strong', labelKey: 'lyrics.desktopLyrics.glowStrong' },
]

async function onDesktopLyricsConfigChange(key: keyof typeof desktopLyricsConfig, value: any) {
  await updateDesktopLyricsConfig(key, value)
}
</script>

<template>
  <v-card class="settings-card" variant="flat" color="surface">
    <div class="card-header">
      <v-icon icon="mdi-monitor-eye" size="18" class="card-icon" />
      <span class="card-title">{{ t('lyrics.desktopLyrics.title') }}</span>
    </div>

    <div class="action-row">
      <v-btn
        color="primary"
        variant="tonal"
        prepend-icon="mdi-monitor-eye"
        @click="toggleDesktopLyrics"
      >
        {{ t('lyrics.desktopLyrics.toggle') }}
      </v-btn>
      <v-checkbox
        :model-value="desktopLyricsLocked"
        :label="t('lyrics.desktopLyrics.locked')"
        hide-details
        density="compact"
        @update:model-value="(v) => toggleDesktopLyricsLock(v ?? false)"
      />
    </div>

    <div class="desktop-lyrics-row">
      <div class="desktop-lyrics-label">{{ t('lyrics.desktopLyrics.fontSize') }}</div>
      <div class="desktop-lyrics-control">
        <v-slider
          :model-value="desktopLyricsConfig.fontSize"
          :min="16"
          :max="64"
          :step="1"
          thumb-label
          hide-details
          color="primary"
          @update:model-value="(v) => onDesktopLyricsConfigChange('fontSize', v)"
        />
      </div>
    </div>

    <div class="desktop-lyrics-row">
      <div class="desktop-lyrics-label">{{ t('lyrics.desktopLyrics.colorCurrent') }}</div>
      <div class="desktop-lyrics-control">
        <input
          type="color"
          class="color-input"
          :value="desktopLyricsConfig.colorCurrent.startsWith('#') ? desktopLyricsConfig.colorCurrent : '#FFFFFF'"
          @input="(e) => onDesktopLyricsConfigChange('colorCurrent', (e.target as HTMLInputElement).value)"
        />
      </div>
    </div>

    <div class="desktop-lyrics-row">
      <div class="desktop-lyrics-label">{{ t('lyrics.desktopLyrics.colorNext') }}</div>
      <div class="desktop-lyrics-control">
        <input
          type="color"
          class="color-input"
          :value="desktopLyricsConfig.colorNext.startsWith('#') ? desktopLyricsConfig.colorNext : '#FFFFFF'"
          @input="(e) => onDesktopLyricsConfigChange('colorNext', (e.target as HTMLInputElement).value)"
        />
      </div>
    </div>

    <div class="desktop-lyrics-row">
      <div class="desktop-lyrics-label">{{ t('lyrics.desktopLyrics.glow') }}</div>
      <div class="mode-toggle desktop-lyrics-control">
        <button
          v-for="opt in glowOptions"
          :key="opt.value"
          :class="['mode-button', { active: desktopLyricsConfig.glowStrength === opt.value }]"
          @click="onDesktopLyricsConfigChange('glowStrength', opt.value)"
        >
          <span class="mode-label">{{ t(opt.labelKey) }}</span>
        </button>
      </div>
    </div>

    <div class="desktop-lyrics-hint">{{ t('lyrics.desktopLyrics.lockHint') }}</div>
  </v-card>
</template>

<style src="../../styles/settings-card.css"></style>
<style scoped>
.desktop-lyrics-row {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-top: 16px;
}

.desktop-lyrics-label {
  flex: 0 0 120px;
  font-size: var(--text-sm);
  color: var(--v-text-secondary);
}

.desktop-lyrics-control {
  flex: 1;
  min-width: 0;
}

.color-input {
  width: 48px;
  height: 32px;
  padding: 0;
  border: 1px solid rgb(var(--v-theme-border));
  border-radius: 8px;
  background: transparent;
  cursor: pointer;
}

.color-input::-webkit-color-swatch-wrapper {
  padding: 2px;
}

.color-input::-webkit-color-swatch {
  border: none;
  border-radius: 6px;
}

.desktop-lyrics-hint {
  margin-top: 16px;
  font-size: var(--text-sm);
  color: var(--v-text-secondary);
  line-height: 1.5;
}
</style>
