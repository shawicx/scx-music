<script setup lang="ts">
import { ref, watch } from 'vue'
import type { Song } from '../../types'
import { useLibraryStore } from '../../stores/library'
import { useI18n } from '../../composables/useI18n'

const props = defineProps<{
  modelValue: boolean
  song: Song | null
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  'renamed': []
}>()

const { t } = useI18n()
const libraryStore = useLibraryStore()

const title = ref('')
const artist = ref('')
const album = ref('')
const loading = ref(false)
const error = ref('')

watch(() => props.modelValue, (open) => {
  if (open && props.song) {
    title.value = props.song.title
    artist.value = props.song.artist
    album.value = props.song.album
    error.value = ''
    loading.value = false
  }
})

async function handleRename() {
  const trimmed = title.value.trim()
  if (!trimmed) return
  if (!props.song) return

  loading.value = true
  error.value = ''

  try {
    await libraryStore.renameSong(
      props.song.id,
      trimmed,
      artist.value.trim() || undefined,
      album.value.trim() || undefined,
    )
    emit('update:modelValue', false)
    emit('renamed')
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <v-dialog :model-value="modelValue" max-width="400" @update:model-value="emit('update:modelValue', $event)">
    <v-card>
      <v-card-title>{{ t('library.rename') }}</v-card-title>
      <v-card-text>
        <v-text-field
          v-model="title"
          :label="t('library.title')"
          density="compact"
          variant="outlined"
          autofocus
          :disabled="loading"
          @keyup.enter="handleRename"
        />
        <v-text-field
          v-model="artist"
          :label="t('library.artist')"
          density="compact"
          variant="outlined"
          :disabled="loading"
          class="mt-2"
        />
        <v-text-field
          v-model="album"
          :label="t('library.album')"
          density="compact"
          variant="outlined"
          :disabled="loading"
          class="mt-2"
        />
        <div v-if="error" class="text-error text-body-2 mt-2">{{ error }}</div>
      </v-card-text>
      <v-card-actions>
        <v-spacer />
        <v-btn variant="text" :disabled="loading" @click="emit('update:modelValue', false)">
          {{ t('common.cancel') }}
        </v-btn>
        <v-btn
          variant="flat"
          color="primary"
          :loading="loading"
          :disabled="!title.trim()"
          @click="handleRename"
        >
          {{ t('common.confirm') }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>
