<script setup lang="ts">
import { ref, nextTick, watch } from 'vue'
import gsap from 'gsap'
import { useI18n } from '../composables/useI18n'
import AppearanceSettings from './settings/AppearanceSettings.vue'
import AudioDeviceSettings from './settings/AudioDeviceSettings.vue'
import AudioEffectSettings from './settings/AudioEffectSettings.vue'
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
    <!-- 页头：返回 + 衬线标题 + 分组 tabbar -->
    <div class="page-head">
      <button class="ph-back" :aria-label="t('common.back')" @click="emit('back')">
        <v-icon icon="mdi-chevron-left" size="18"></v-icon>
      </button>
      <h1 class="ph-title">{{ t('settings.title') }}</h1>
      <nav class="tabbar" role="tablist" :aria-label="t('settings.title')">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          class="tab-btn"
          role="tab"
          :aria-selected="activeTab === tab.id"
          @click="activeTab = tab.id"
        >
          <v-icon :icon="tab.icon" size="14"></v-icon>
          <span>{{ t(tab.label) }}</span>
        </button>
      </nav>
    </div>

    <div ref="contentRef" class="settings-content">
      <AppearanceSettings v-if="activeTab === 'appearance'" />
      <template v-else-if="activeTab === 'audio'">
        <AudioDeviceSettings />
        <AudioEffectSettings />
      </template>
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

/* 页头 */
.page-head {
  display: flex;
  align-items: flex-end;
  gap: 12px;
  padding: 20px 28px 14px;
  border-bottom: var(--hairline-w) solid var(--hairline);
  flex-shrink: 0;
  flex-wrap: wrap;
  row-gap: 10px;
}
.ph-back {
  width: 30px;
  height: 30px;
  border-radius: 8px;
  margin-bottom: 4px;
  display: grid;
  place-items: center;
  color: var(--text-2);
  flex-shrink: 0;
  background: none;
  border: none;
  cursor: pointer;
}
.ph-back:hover {
  background: var(--surface-2);
  color: var(--text-1);
}
.ph-title {
  font-family: var(--font-display);
  font-size: 26px;
  font-weight: 560;
  line-height: 1.15;
  color: var(--text-1);
}

/* 分组 tabbar */
.tabbar {
  margin-left: auto;
  display: flex;
  gap: 2px;
  padding: 2px;
  border-radius: 9px;
  background: var(--surface-2);
  border: var(--hairline-w) solid var(--hairline);
  flex-wrap: wrap;
}
.tab-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 11px;
  border-radius: 7px;
  background: transparent;
  color: var(--text-2);
  cursor: pointer;
  font-size: 12px;
  font-family: inherit;
  white-space: nowrap;
  border: none;
  transition: background 0.15s, color 0.15s;
}
.tab-btn:hover {
  color: var(--text-1);
}
.tab-btn[aria-selected='true'] {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text-1);
  font-weight: 600;
  box-shadow:
    0 1px 3px rgba(0, 0, 0, 0.3),
    inset 0 1px 0 rgba(255, 255, 255, 0.06);
}
:global(.scx-light) .tab-btn[aria-selected='true'] {
  background: #ffffff;
  box-shadow: 0 1px 4px rgba(20, 20, 24, 0.14);
}

.settings-content {
  flex: 1;
  overflow-y: auto;
  padding: 20px 28px 28px;
  max-width: 860px;
}
</style>
