<script setup lang="ts">
import { ref, nextTick, watch } from 'vue'
import gsap from 'gsap'
import { useI18n } from '../composables/useI18n'
import AppearanceSettings from './settings/AppearanceSettings.vue'
import AudioDeviceSettings from './settings/AudioDeviceSettings.vue'
import DesktopLyricsSettings from './settings/DesktopLyricsSettings.vue'
import ShortcutSettings from './settings/ShortcutSettings.vue'
import DataManagementSettings from './settings/DataManagementSettings.vue'
import StartupSettings from './settings/StartupSettings.vue'

const emit = defineEmits<{ back: [] }>()

const { t } = useI18n()

type TabId = 'appearance' | 'audio' | 'lyrics' | 'shortcuts' | 'data' | 'startup'
const activeTab = ref<TabId>('appearance')

const tabs: { id: TabId; icon: string; label: string }[] = [
  { id: 'appearance', icon: 'mdi-palette', label: 'settings.tabs.appearance' },
  { id: 'audio', icon: 'mdi-speaker', label: 'settings.tabs.audio' },
  { id: 'lyrics', icon: 'mdi-monitor-eye', label: 'settings.tabs.lyrics' },
  { id: 'shortcuts', icon: 'mdi-keyboard', label: 'settings.tabs.shortcuts' },
  { id: 'data', icon: 'mdi-database', label: 'settings.tabs.data' },
  { id: 'startup', icon: 'mdi-power', label: 'settings.tabs.startup' },
]

/** 内容区容器引用：tab 切换时对新内容做 GSAP 淡入上移动画。 */
const contentRef = ref<HTMLElement | null>(null)

watch(activeTab, async () => {
  await nextTick()
  if (contentRef.value) {
    gsap.fromTo(
      contentRef.value,
      { opacity: 0, y: 12 },
      { opacity: 1, y: 0, duration: 0.28, ease: 'expo.out' },
    )
  }
})
</script>

<template>
  <div class="settings-view">
    <div class="settings-header">
      <v-btn icon variant="plain" size="small" @click="emit('back')">
        <v-icon icon="mdi-arrow-left" size="20" />
      </v-btn>
      <h2 class="settings-title">{{ t('settings.title') }}</h2>
    </div>

    <nav class="settings-tabs">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        :class="['tab-btn', { active: activeTab === tab.id }]"
        @click="activeTab = tab.id"
      >
        <v-icon :icon="tab.icon" size="18" />
        <span>{{ t(tab.label) }}</span>
      </button>
    </nav>

    <div ref="contentRef" class="settings-content">
      <AppearanceSettings v-if="activeTab === 'appearance'" />
      <AudioDeviceSettings v-else-if="activeTab === 'audio'" />
      <DesktopLyricsSettings v-else-if="activeTab === 'lyrics'" />
      <ShortcutSettings v-else-if="activeTab === 'shortcuts'" />
      <DataManagementSettings v-else-if="activeTab === 'data'" />
      <StartupSettings v-else-if="activeTab === 'startup'" />
    </div>
  </div>
</template>

<style scoped>
.settings-view {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.settings-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 16px 32px;
  border-bottom: 1px solid var(--v-border-color);
}

.settings-title {
  font-size: var(--text-xl);
  font-weight: 700;
  color: rgb(var(--v-theme-on-background));
}

.settings-tabs {
  display: flex;
  gap: 4px;
  padding: 8px 32px;
  border-bottom: 1px solid var(--v-border-color);
  overflow-x: auto;
  flex-shrink: 0;
}

.tab-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border-radius: 8px;
  border: 1px solid transparent;
  background: transparent;
  color: var(--v-text-secondary);
  cursor: pointer;
  font-size: var(--text-sm);
  white-space: nowrap;
  transition: all 0.15s ease;
}

.tab-btn:hover {
  background: var(--v-accent-bg);
}

.tab-btn.active {
  background: rgb(var(--v-theme-primary));
  color: rgb(var(--v-theme-on-primary));
  font-weight: 600;
}

.settings-content {
  flex: 1;
  overflow-y: auto;
  padding: 32px;
}
</style>
