<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useMouse } from '@vueuse/core'
import { useLibraryStore } from '../stores/library'
import { useToast } from '../composables/useToast'
import { useI18n } from '../composables/useI18n'
import { useImportExport } from '../composables/useImportExport'

defineProps<{ activeView: string }>()
const emit = defineEmits<{ navigate: [view: string] }>()

const libraryStore = useLibraryStore()
const { showSuccess, showWarning } = useToast()
const { t } = useI18n()
const { exportPlaylist } = useImportExport()

const {
  playlists,
  activePlaylistId,
} = storeToRefs(libraryStore)

const {
  addPlaylist,
  renamePlaylist,
  deletePlaylist,
  clearPlaylist,
  importToPlaylist,
  setActivePlaylist,
} = libraryStore

const showAddDialog = ref(false)
const newPlaylistName = ref('')
const contextMenu = ref<{ show: boolean; x: number; y: number; playlistId: string; playlistName: string }>({
  show: false, x: 0, y: 0, playlistId: '', playlistName: '',
})
const showRenameDialog = ref(false)
const renameValue = ref('')
const renamingPlaylistId = ref('')
const isImporting = ref<string | null>(null)

// 拖拽调整宽度
const { x: mouseX } = useMouse()
const isDragging = ref(false)
const dragStartX = ref(0)
const dragStartWidth = ref(200)
const sidebarWidth = ref(200)
const minWidth = 160
const maxWidth = 400

function startDrag(e: MouseEvent) {
  e.preventDefault()
  isDragging.value = true
  dragStartX.value = e.clientX
  dragStartWidth.value = sidebarWidth.value
}

// 监听鼠标移动来调整宽度
const currentWidth = computed(() => {
  if (!isDragging.value) return sidebarWidth.value
  const delta = mouseX.value - dragStartX.value
  return Math.min(Math.max(dragStartWidth.value + delta, minWidth), maxWidth)
})

function stopDrag() {
  if (isDragging.value) {
    sidebarWidth.value = currentWidth.value
    isDragging.value = false
  }
}

// 监听鼠标释放
onMounted(() => {
  document.addEventListener('mouseup', stopDrag)
})

onUnmounted(() => {
  document.removeEventListener('mouseup', stopDrag)
})

function handlePlaylistClick(id: string) {
  setActivePlaylist(id)
  emit('navigate', 'library')
}

function handleContextMenu(e: MouseEvent, pl: { id: string; name: string }) {
  e.preventDefault()
  contextMenu.value = { show: true, x: e.clientX, y: e.clientY, playlistId: pl.id, playlistName: pl.name }
}

function closeContextMenu() {
  contextMenu.value.show = false
}

async function handleAddPlaylist() {
  const name = newPlaylistName.value.trim()
  if (!name) return
  const id = await addPlaylist(name)
  newPlaylistName.value = ''
  showAddDialog.value = false
  setActivePlaylist(id)
  showSuccess(t('toast.playlistCreated', { name }))
}

function startRename() {
  renameValue.value = contextMenu.value.playlistName
  renamingPlaylistId.value = contextMenu.value.playlistId
  contextMenu.value.show = false
  showRenameDialog.value = true
}

function handleRename() {
  const name = renameValue.value.trim()
  if (!name) return
  renamePlaylist(renamingPlaylistId.value, name)
  showRenameDialog.value = false
  showSuccess(t('toast.playlistRenamed'))
}

function handleClearPlaylist() {
  const pid = contextMenu.value.playlistId
  contextMenu.value.show = false
  clearPlaylist(pid)
}

function handleDelete() {
  const { playlistId, playlistName } = contextMenu.value
  if (playlistId === 'fav') return
  contextMenu.value.show = false
  deletePlaylist(playlistId)
  showWarning(t('toast.playlistDeletedMsg', { name: playlistName }))
}

async function handleImport() {
  const pid = contextMenu.value.playlistId
  contextMenu.value.show = false
  isImporting.value = pid
  try {
    const count = await importToPlaylist(pid)
    if (count !== undefined) {
      if (count > 0) {
        showSuccess(t('toast.songsImported', { count }))
      } else {
        showWarning(t('toast.noSongsFound'))
      }
    }
  } finally {
    isImporting.value = null
  }
}

function handleExportPlaylist() {
  const { playlistId, playlistName } = contextMenu.value
  contextMenu.value.show = false
  exportPlaylist(playlistId, playlistName)
}
</script>

<template>
  <aside
    class="sidebar"
    :class="{ dragging: isDragging }"
    :style="{ width: `${currentWidth}px` }"
    @click="closeContextMenu"
  >
    <!-- 拖拽手柄 -->
    <div class="resize-handle" @mousedown.stop="startDrag" />
    <!-- Logo -->
    <div class="sidebar-logo">
      <span class="logo-icon"><v-icon icon="mdi-music" size="14"></v-icon></span>
      <span class="logo-text">scx-music</span>
    </div>

    <!-- Playlists -->
    <div class="sidebar-section">
      <div class="section-header">
        <span class="section-title">{{ t('sidebar.playlists') }}</span>
        <v-btn icon size="x-small" variant="plain" density="compact" @click="showAddDialog = true">
          <v-icon icon="mdi-plus" size="20"></v-icon>
        </v-btn>
      </div>
      <v-list density="comfortable" bg-color="transparent" class="playlist-list" color="secondary">
        <v-list-item
          v-for="pl in playlists"
          :key="pl.id"
          :title="pl.name"
          rounded="lg"
          :active="pl.id === activePlaylistId && activeView === 'library'"
          @click="handlePlaylistClick(pl.id)"
          @contextmenu="handleContextMenu($event, pl)"
        >
          <template #append>
            <v-progress-circular
              v-if="isImporting === pl.id"
              size="14"
              width="2"
              indeterminate
              color="secondary"
            />
          </template>
        </v-list-item>
      </v-list>
    </div>

    <!-- Settings -->
    <div class="sidebar-bottom">
      <v-btn
        block
        variant="text"
        prepend-icon="mdi-chart-timeline-variant"
        :class="['settings-btn', { active: activeView === 'stats' }]"
        @click="emit('navigate', 'stats')"
      >
        {{ t('sidebar.listeningStats') }}
      </v-btn>
      <v-btn
        block
        variant="text"
        prepend-icon="mdi-chart-bar"
        :class="['settings-btn', { active: activeView === 'analysis' }]"
        @click="emit('navigate', 'analysis')"
      >
        {{ t('sidebar.analysis') }}
      </v-btn>
      <v-btn
        block
        variant="text"
        prepend-icon="mdi-cog"
        :class="['settings-btn', { active: activeView === 'settings' }]"
        @click="emit('navigate', 'settings')"
      >
        {{ t('sidebar.settings') }}
      </v-btn>
    </div>

    <!-- Context menu -->
    <v-menu
      v-model="contextMenu.show"
      :target="[contextMenu.x, contextMenu.y]"
      :close-on-content-click="false"
    >
      <v-list density="compact" min-width="140">
        <v-list-item prepend-icon="mdi-folder-open" :title="t('sidebar.importFolder')" @click="handleImport" />
        <v-list-item prepend-icon="mdi-export" :title="t('sidebar.exportPlaylist')" @click="handleExportPlaylist" />
        <v-list-item prepend-icon="mdi-playlist-remove" :title="t('sidebar.clearPlaylist')" @click="handleClearPlaylist" />
        <v-list-item v-if="contextMenu.playlistId !== 'fav'" prepend-icon="mdi-pencil" :title="t('sidebar.rename')" @click="startRename" />
        <v-list-item v-if="contextMenu.playlistId !== 'fav'" prepend-icon="mdi-delete" :title="t('common.delete')" @click="handleDelete" />
      </v-list>
    </v-menu>

    <!-- Add dialog -->
    <v-dialog v-model="showAddDialog" width="400">
      <v-card>
        <v-card-title>{{ t('sidebar.newPlaylist') }}</v-card-title>
        <v-card-text>
          <v-text-field
            v-model="newPlaylistName"
            :label="t('sidebar.playlistName')"
            density="compact"
            variant="outlined"
            hide-details
            autofocus
            @keyup.enter="handleAddPlaylist"
          />
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="text" @click="showAddDialog = false">{{ t('common.cancel') }}</v-btn>
          <v-btn variant="flat" color="primary" @click="handleAddPlaylist">{{ t('common.create') }}</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Rename dialog -->
    <v-dialog v-model="showRenameDialog" width="400">
      <v-card>
        <v-card-title>{{ t('sidebar.renamePlaylist') }}</v-card-title>
        <v-card-text>
          <v-text-field
            v-model="renameValue"
            :label="t('sidebar.playlistName')"
            density="compact"
            variant="outlined"
            hide-details
            autofocus
            @keyup.enter="handleRename"
          />
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="text" @click="showRenameDialog = false">{{ t('common.cancel') }}</v-btn>
          <v-btn variant="flat" color="primary" @click="handleRename">{{ t('common.confirm') }}</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </aside>
</template>

<style scoped>
.sidebar {
  background: var(--glass-bg);
  backdrop-filter: blur(var(--glass-blur));
  border-right: 1px solid var(--glass-border);
  display: flex;
  flex-direction: column;
  padding: 16px 0;
  flex-shrink: 0;
  overflow-y: auto;
  position: relative;
}

.sidebar.dragging {
  user-select: none;
}

.resize-handle {
  position: absolute;
  right: 0;
  top: 0;
  bottom: 0;
  width: 4px;
  cursor: col-resize;
  transition: background-color 0.2s;
}

.resize-handle:hover {
  background-color: rgb(var(--v-theme-primary));
}

.sidebar-logo {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 16px 20px;
}

.logo-icon {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--v-gradient-brand);
  border-radius: 8px;
  flex-shrink: 0;
}

.logo-text {
  font-size: var(--text-md);
  font-weight: 700;
  background: var(--v-gradient-brand-text);
  -webkit-background-clip: text;
  background-clip: text;
  -webkit-text-fill-color: transparent;
}

.sidebar-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 0 8px;
  min-height: 0;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 12px 8px;
  font-size: 1.5rem;
}

.section-title {
  font-weight: 600;
  text-transform: uppercase;
  color: var(--v-text-muted);
  letter-spacing: 0.5px;
}

.playlist-list { padding: 0; }
.playlist-list :deep(.v-list-item) { padding-left: 16px; }

.sidebar-bottom { padding: 0 12px; }
.settings-btn {
  letter-spacing: 0;
  color: var(--v-text-secondary);
}
.settings-btn.active {
  color: rgb(var(--v-theme-primary));
  background: var(--v-accent-bg);
}
</style>
