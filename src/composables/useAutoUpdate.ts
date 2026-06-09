import { ref } from 'vue'
import { check } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'

export type UpdateState = 'idle' | 'available' | 'downloading' | 'ready' | 'error'

const showDialog = ref(false)
const updateState = ref<UpdateState>('idle')
const newVersion = ref('')
const downloadProgress = ref(0)
const errorMessage = ref('')

let dismissed = false
let totalBytes = 0
let downloadedBytes = 0

export function useAutoUpdate() {
  function checkForUpdate() {
    if (!import.meta.env.PROD) return
    if (dismissed) return

    check()
      .then((update) => {
        if (!update?.available) return
        newVersion.value = update.version
        updateState.value = 'available'
        showDialog.value = true
      })
      .catch(() => {
        // 静默忽略检查失败
      })
  }

  async function downloadAndInstall() {
    try {
      const update = await check()
      if (!update?.available) return

      updateState.value = 'downloading'
      downloadProgress.value = 0
      errorMessage.value = ''
      totalBytes = 0
      downloadedBytes = 0

      await update.downloadAndInstall((event) => {
        switch (event.event) {
          case 'Started':
            totalBytes = event.data.contentLength ?? 0
            break
          case 'Progress':
            downloadedBytes += event.data.chunkLength
            if (totalBytes > 0) {
              downloadProgress.value = Math.round((downloadedBytes / totalBytes) * 100)
            }
            break
          case 'Finished':
            updateState.value = 'ready'
            break
        }
      })
    } catch (e) {
      updateState.value = 'error'
      errorMessage.value = e instanceof Error ? e.message : String(e)
    }
  }

  function dismiss() {
    dismissed = true
    showDialog.value = false
    updateState.value = 'idle'
  }

  async function restart() {
    await relaunch()
  }

  function startCheck() {
    setTimeout(checkForUpdate, 3000)
  }

  return {
    showDialog,
    updateState,
    newVersion,
    downloadProgress,
    errorMessage,
    dismiss,
    downloadAndInstall,
    restart,
    startCheck,
  }
}
