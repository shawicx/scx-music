<script setup lang="ts">
import { ref } from 'vue'
import { useLibrary } from '../composables/useLibrary'

defineProps<{ activeView: string }>()
const emit = defineEmits<{ navigate: [view: string] }>()

const {
  playlists,
  activePlaylistId,
  addPlaylist,
  renamePlaylist,
  deletePlaylist,
  importToPlaylist,
  setActivePlaylist,
} = useLibrary()

const showAddDialog = ref(false)
const newPlaylistName = ref('')
const showSnackbar = ref(false)
const snackbarText = ref('')
const snackbarColor = ref('success')
const contextMenu = ref<{ show: boolean; x: number; y: number; playlistId: string; playlistName: string }>({
  show: false, x: 0, y: 0, playlistId: '', playlistName: '',
})
const showRenameDialog = ref(false)
const renameValue = ref('')
const renamingPlaylistId = ref('')
const isImporting = ref<string | null>(null)

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
  snackbarText.value = `歌单「${name}」已创建`
  snackbarColor.value = 'success'
  showSnackbar.value = true
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
}

function handleDelete() {
  const { playlistId, playlistName } = contextMenu.value
  contextMenu.value.show = false
  deletePlaylist(playlistId)
  snackbarText.value = `歌单「${playlistName}」已删除`
  snackbarColor.value = 'warning'
  showSnackbar.value = true
}

async function handleImport() {
  const pid = contextMenu.value.playlistId
  contextMenu.value.show = false
  isImporting.value = pid
  try {
    const count = await importToPlaylist(pid)
    if (count !== undefined) {
      snackbarText.value = `已导入 ${count} 首歌曲`
      snackbarColor.value = count > 0 ? 'success' : 'warning'
      showSnackbar.value = true
    }
  } finally {
    isImporting.value = null
  }
}

async function handleImportActive() {
  if (!activePlaylistId.value) return
  isImporting.value = activePlaylistId.value
  try {
    const count = await importToPlaylist(activePlaylistId.value)
    if (count !== undefined) {
      snackbarText.value = `已导入 ${count} 首歌曲`
      snackbarColor.value = count > 0 ? 'success' : 'warning'
      showSnackbar.value = true
    }
  } finally {
    isImporting.value = null
  }
}
</script>

<template>
  <aside class="sidebar" @click="closeContextMenu">
    <!-- Logo -->
    <div class="sidebar-logo">
      <span class="logo-icon"><v-icon icon="mdi-music" size="14"></v-icon></span>
      <span class="logo-text">scx-music</span>
    </div>

    <!-- Playlists -->
    <div class="sidebar-section">
      <div class="section-header">
        <span class="section-title">歌单</span>
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

    <!-- Import for active playlist -->
    <div v-if="activePlaylistId" class="import-wrapper">
      <v-btn
        block
        variant="outlined"
        :disabled="isImporting !== null"
        prepend-icon="mdi-folder-open"
        class="import-btn"
        @click="handleImportActive"
      >
        {{ isImporting ? '扫描中...' : '导入到当前歌单' }}
      </v-btn>
    </div>

    <!-- Settings -->
    <div class="sidebar-bottom">
      <v-btn
        block
        variant="text"
        prepend-icon="mdi-cog"
        :class="['settings-btn', { active: activeView === 'settings' }]"
        @click="emit('navigate', 'settings')"
      >
        设置
      </v-btn>
    </div>

    <!-- Context menu -->
    <v-menu
      v-model="contextMenu.show"
      :target="[contextMenu.x, contextMenu.y]"
      :close-on-content-click="false"
    >
      <v-list density="compact" min-width="140">
        <v-list-item prepend-icon="mdi-folder-open" title="导入文件夹" @click="handleImport" />
        <v-list-item prepend-icon="mdi-pencil" title="重命名" @click="startRename" />
        <v-list-item prepend-icon="mdi-delete" title="删除" @click="handleDelete" />
      </v-list>
    </v-menu>

    <!-- Snackbar -->
    <v-snackbar
      v-model="showSnackbar"
      :color="snackbarColor"
      :timeout="3000"
      location="bottom"
    >
      {{ snackbarText }}
    </v-snackbar>

    <!-- Add dialog -->
    <v-dialog v-model="showAddDialog" width="400">
      <v-card style="padding: 1rem;">
        <v-card-title>新建歌单</v-card-title>
        <v-card-text>
          <v-text-field
            v-model="newPlaylistName"
            label="歌单名称"
            density="compact"
            variant="outlined"
            hide-details
            autofocus
            @keyup.enter="handleAddPlaylist"
          />
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="text" @click="showAddDialog = false">取消</v-btn>
          <v-btn variant="flat" color="primary" @click="handleAddPlaylist">创建</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Rename dialog -->
    <v-dialog v-model="showRenameDialog" width="400">
      <v-card style="padding: 1rem;">
        <v-card-title>重命名歌单</v-card-title>
        <v-card-text>
          <v-text-field
            v-model="renameValue"
            label="歌单名称"
            density="compact"
            variant="outlined"
            hide-details
            autofocus
            @keyup.enter="handleRename"
          />
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="text" @click="showRenameDialog = false">取消</v-btn>
          <v-btn variant="flat" color="primary" @click="handleRename">确定</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 200px;
  background: rgb(var(--v-theme-surface-bright));
  border-right: 1px solid var(--v-border-color);
  display: flex;
  flex-direction: column;
  padding: 16px 0;
  flex-shrink: 0;
  overflow-y: auto;
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

.import-wrapper { padding: 4px 12px 0; }
.import-btn {
  border-color: rgb(var(--v-theme-secondary));
  color: rgb(var(--v-theme-secondary));
  background: var(--v-accent-bg);
  text-transform: none;
}
.import-btn:hover { background: var(--v-accent-glow); }

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
