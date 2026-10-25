<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useMouse } from '@vueuse/core'
import { useLibraryStore } from '../stores/library'
import { useToast } from '../composables/useToast'
import { useI18n } from '../composables/useI18n'
import { useImportExport } from '../composables/useImportExport'

defineProps<{ activeView: string }>()
const emit = defineEmits<{ navigate: [view: string]; resize: [width: number] }>()

const libraryStore = useLibraryStore()
const { showSuccess, showWarning } = useToast()
const { t } = useI18n()
const { exportPlaylist } = useImportExport()

const {
  songs,
  playlists,
  playlistSongs,
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

// 'fav'（我的收藏）显示在「资料库」组，其余歌单归「歌单」组
const customPlaylists = computed(() => playlists.value.filter((p) => p.id !== 'fav'))
const allCount = computed(() => songs.value.length)
const favCount = computed(() => playlistSongs.value['fav']?.length ?? 0)
const plCount = (id: string) => playlistSongs.value[id]?.length ?? 0

const showAddDialog = ref(false)
const newPlaylistName = ref('')
const contextMenu = ref<{ show: boolean; x: number; y: number; playlistId: string; playlistName: string }>({
  show: false, x: 0, y: 0, playlistId: '', playlistName: '',
})
const showRenameDialog = ref(false)
const renameValue = ref('')
const renamingPlaylistId = ref('')
const isImporting = ref<string | null>(null)

// 拖拽调整宽度（写回 App.vue 的 --sb-w 网格列）
const { x: mouseX } = useMouse()
const isDragging = ref(false)
const dragStartX = ref(0)
const dragStartWidth = ref(236)
const sidebarWidth = ref(236)
const minWidth = 208
const maxWidth = 320

function startDrag(e: MouseEvent) {
  e.preventDefault()
  isDragging.value = true
  dragStartX.value = e.clientX
  dragStartWidth.value = sidebarWidth.value
}

const currentWidth = computed(() => {
  if (!isDragging.value) return sidebarWidth.value
  const delta = mouseX.value - dragStartX.value
  return Math.min(Math.max(dragStartWidth.value + delta, minWidth), maxWidth)
})

function stopDrag() {
  if (isDragging.value) {
    sidebarWidth.value = currentWidth.value
    emit('resize', sidebarWidth.value)
    isDragging.value = false
  }
}

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
  <aside class="sidebar" :class="{ dragging: isDragging }" @click="closeContextMenu">
    <div class="resize-handle" @mousedown.stop="startDrag" />

    <!-- 品牌 -->
    <div class="sb-brand">
      <span class="mark"><v-icon icon="mdi-music" size="13"></v-icon></span>
      <span class="name">scx-music</span>
    </div>

    <!-- 资料库 -->
    <nav class="sb-group" :aria-label="t('sidebar.library')">
      <div class="sb-group-label">{{ t('sidebar.library') }}</div>
      <button
        class="sb-item"
        :aria-current="activePlaylistId === 'all' && activeView === 'library'"
        @click="handlePlaylistClick('all')"
      >
        <v-icon icon="mdi-music" size="17"></v-icon>
        <span>{{ t('sidebar.allSongs') }}</span>
        <span class="count num">{{ allCount }}</span>
      </button>
      <button
        class="sb-item"
        :aria-current="activePlaylistId === 'fav' && activeView === 'library'"
        @click="handlePlaylistClick('fav')"
      >
        <v-icon icon="mdi-heart-outline" size="17"></v-icon>
        <span>{{ t('sidebar.favorites') }}</span>
        <span class="count num">{{ favCount }}</span>
      </button>
    </nav>

    <!-- 歌单 -->
    <nav class="sb-group sb-group-playlists" :aria-label="t('sidebar.playlists')">
      <div class="sb-group-label">{{ t('sidebar.playlists') }}</div>
      <button
        v-for="pl in customPlaylists"
        :key="pl.id"
        class="sb-item"
        :aria-current="pl.id === activePlaylistId && activeView === 'library'"
        @click="handlePlaylistClick(pl.id)"
        @contextmenu="handleContextMenu($event, pl)"
      >
        <v-icon icon="mdi-playlist-music" size="17"></v-icon>
        <span class="pl-name">{{ pl.name }}</span>
        <v-progress-circular
          v-if="isImporting === pl.id"
          size="12"
          width="1.5"
          indeterminate
          class="count"
        />
        <span v-else class="count num">{{ plCount(pl.id) }}</span>
      </button>
      <button class="sb-new" @click="showAddDialog = true">
        <v-icon icon="mdi-plus" size="15"></v-icon>
        <span>{{ t('sidebar.newPlaylist') }}</span>
      </button>
    </nav>

    <!-- 底部入口 -->
    <nav class="sb-bottom" aria-label="更多">
      <button
        class="sb-item"
        :aria-current="activeView === 'stats'"
        @click="emit('navigate', 'stats')"
      >
        <v-icon icon="mdi-chart-timeline-variant" size="17"></v-icon>
        <span>{{ t('sidebar.listeningStats') }}</span>
      </button>
      <button
        class="sb-item"
        :aria-current="activeView === 'analysis'"
        @click="emit('navigate', 'analysis')"
      >
        <v-icon icon="mdi-chart-bar" size="17"></v-icon>
        <span>{{ t('sidebar.analysis') }}</span>
      </button>
      <button
        class="sb-item"
        :aria-current="activeView === 'settings'"
        @click="emit('navigate', 'settings')"
      >
        <v-icon icon="mdi-cog" size="17"></v-icon>
        <span>{{ t('sidebar.settings') }}</span>
      </button>
    </nav>

    <!-- Context menu -->
    <v-menu
      v-model="contextMenu.show"
      :target="[contextMenu.x, contextMenu.y]"
      :close-on-content-click="false"
    >
      <v-list density="compact" min-width="150">
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
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--glass-bg);
  -webkit-backdrop-filter: blur(var(--glass-blur)) saturate(1.55);
  backdrop-filter: blur(var(--glass-blur)) saturate(1.55);
  border-right: var(--hairline-w) solid var(--hairline);
  padding: 6px 10px 10px;
  position: relative;
  z-index: 30;
  overflow: hidden;
}
/* 浅色：极简实底，不用玻璃 */
:global(.scx-light) .sidebar {
  background: var(--surface-0);
  -webkit-backdrop-filter: none;
  backdrop-filter: none;
}

.sidebar.dragging {
  user-select: none;
}

.resize-handle {
  position: absolute;
  right: -2px;
  top: 0;
  bottom: 0;
  width: 5px;
  cursor: col-resize;
  z-index: 5;
}
.resize-handle:hover {
  background: rgb(var(--accent-rgb) / 0.35);
}

.sb-brand {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 8px 8px 14px;
}
/* macOS 原生标题栏在场（Overlay 被系统忽略）时无自绘标题行，侧栏从 y=0 开始，
 * 品牌行需让开顶部 28pt 红绿灯区，落到「资料库」组上方 */
:global(.native-mac-bar) .sb-brand {
  padding-top: 34px;
}
.sb-brand .mark {
  width: 22px;
  height: 22px;
  border-radius: 7px;
  background: linear-gradient(135deg, var(--accent), rgb(var(--accent-rgb) / 0.55));
  display: grid;
  place-items: center;
  color: var(--on-accent);
  flex-shrink: 0;
}
.sb-brand .name {
  font-size: 13px;
  font-weight: 650;
  letter-spacing: 0.01em;
}

.sb-group {
  padding: 10px 4px 4px;
}
.sb-group-playlists {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
  scrollbar-width: thin;
}
.sb-group-label {
  font-size: 10.5px;
  font-weight: 640;
  letter-spacing: 0.09em;
  color: var(--text-3);
  padding: 6px 8px;
}

.sb-item {
  display: flex;
  align-items: center;
  gap: 9px;
  width: 100%;
  padding: 6px 8px;
  border-radius: 7px;
  color: var(--text-2);
  font-size: 13px;
  text-align: left;
  transition: background 0.14s, color 0.14s;
  cursor: pointer;
  background: none;
  border: none;
  font-family: inherit;
}
.sb-item > :deep(.v-icon) {
  flex-shrink: 0;
  opacity: 0.8;
}
.sb-item .pl-name {
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.sb-item:hover {
  background: var(--surface-2);
  color: var(--text-1);
}
.sb-item[aria-current='true'] {
  background: rgb(var(--accent-rgb) / 0.16);
  color: var(--text-1);
}
.sb-item[aria-current='true'] :deep(.v-icon) {
  color: var(--accent);
  opacity: 1;
}
.sb-item .count {
  margin-left: auto;
  font-size: 11px;
  color: var(--text-3);
  flex-shrink: 0;
}

.sb-new {
  display: flex;
  align-items: center;
  gap: 9px;
  width: 100%;
  padding: 6px 8px;
  border-radius: 7px;
  color: var(--text-3);
  font-size: 12.5px;
  margin-top: 2px;
  background: none;
  border: none;
  cursor: pointer;
  font-family: inherit;
  transition: background 0.14s, color 0.14s;
}
.sb-new:hover {
  background: var(--surface-2);
  color: var(--text-1);
}

.sb-bottom {
  margin-top: auto;
  border-top: var(--hairline-w) solid var(--hairline);
  padding-top: 6px;
}
</style>
