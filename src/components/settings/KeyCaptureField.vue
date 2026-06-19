<script setup lang="ts">
import { ref, onUnmounted } from 'vue'
import { formatCombo, buildComboFromEvent, detectPlatform } from '../../utils/keycode'
import { useI18n } from '../../composables/useI18n'

interface Props {
  actionId: string
  actionLabel: string
  combo: string
  enabled: boolean
}

const props = defineProps<Props>()
const emit = defineEmits<{
  rebind: [actionId: string, newCombo: string]
  toggleEnabled: [actionId: string, enabled: boolean]
}>()

const { t } = useI18n()
const platform = detectPlatform()

const isCapturing = ref(false)
const capturedCombo = ref('')

function startCapture() {
  isCapturing.value = true
  capturedCombo.value = ''
  window.addEventListener('keydown', handleKey, true)
}

function stopCapture() {
  isCapturing.value = false
  capturedCombo.value = ''
  window.removeEventListener('keydown', handleKey, true)
}

function handleKey(e: KeyboardEvent) {
  e.preventDefault()
  e.stopPropagation()

  if (e.key === 'Escape') {
    stopCapture()
    return
  }

  const combo = buildComboFromEvent(e)
  if (!combo) return  // modifier-only press, wait for full combo

  // Capture complete: keep capture mode to show preview + save button, but stop listening
  capturedCombo.value = combo
  window.removeEventListener('keydown', handleKey, true)
}

onUnmounted(stopCapture)

function onSave() {
  if (!capturedCombo.value) return
  emit('rebind', props.actionId, capturedCombo.value)
  stopCapture()
}

function onClearBinding() {
  emit('rebind', props.actionId, '')
  stopCapture()
}

function onToggleEnabled() {
  emit('toggleEnabled', props.actionId, !props.enabled)
}
</script>

<template>
  <v-list-item>
    <template #prepend>
      <v-tooltip location="top">
        <template #activator="{ props: activatorProps }">
          <v-icon v-bind="activatorProps" size="small">mdi-keyboard-outline</v-icon>
        </template>
        {{ actionId }}
      </v-tooltip>
    </template>

    <v-list-item-title>{{ actionLabel }}</v-list-item-title>
    <v-list-item-subtitle v-if="combo" class="text-monospace">
      {{ formatCombo(combo, platform) }}
    </v-list-item-subtitle>
    <v-list-item-subtitle v-else class="text-disabled">
      {{ t('settings.shortcuts.notBound') }}
    </v-list-item-subtitle>

    <template #append>
      <div class="d-flex align-center gap-2">
        <v-switch
          :model-value="enabled"
          density="compact"
          color="primary"
          hide-details
          @update:model-value="onToggleEnabled"
        />

        <!-- Default state: Rebind / Clear buttons -->
        <template v-if="!isCapturing">
          <v-btn size="small" variant="text" @click="startCapture">
            {{ t('settings.shortcuts.rebind') }}
          </v-btn>
          <v-btn
            v-if="combo"
            size="small"
            variant="text"
            color="error"
            @click="onClearBinding"
          >
            {{ t('settings.shortcuts.clear') }}
          </v-btn>
        </template>

        <!-- Capture state: prompt / preview + save + cancel -->
        <template v-else>
          <span v-if="!capturedCombo" class="text-caption text-warning">
            {{ t('settings.shortcuts.capture.placeholder') }}
          </span>
          <template v-else>
            <span class="text-caption font-weight-bold">
              {{ formatCombo(capturedCombo, platform) }}
            </span>
            <v-btn size="small" variant="text" color="primary" @click="onSave">
              {{ t('settings.shortcuts.capture.save') }}
            </v-btn>
          </template>
          <v-btn size="small" variant="text" @click="stopCapture">
            {{ t('settings.shortcuts.capture.cancel') }}
          </v-btn>
        </template>
      </div>
    </template>
  </v-list-item>
</template>

<style scoped>
.gap-2 { gap: 8px; }
.text-monospace { font-family: ui-monospace, SFMono-Regular, monospace; }
</style>
