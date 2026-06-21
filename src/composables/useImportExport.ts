import { invokeCommand } from '../utils/errorHandler'
import { useToast } from './useToast'
import { useI18n } from './useI18n'
import type { SaveDialogOptions, OpenDialogOptions } from '@tauri-apps/plugin-dialog'

interface ImportResult {
  songsImported: number
  playlistsImported: number
  lyricsImported: number
}

/**
 * 文件对话框 + IPC 调用 + toast 的统一封装。
 * @param type 'save'（导出）或 'open'（导入）
 * @param dialogOptions save()/open() 配置
 * @param action 拿到 path 后执行的 IPC 调用
 * @param successMsg 成功 toast 文案
 * @param errorMsg 失败 toast 文案
 * @param showSuccess 成功 toast 函数
 * @param showError 失败 toast 函数
 */
async function withFileDialog(
  type: 'save' | 'open',
  dialogOptions: SaveDialogOptions | OpenDialogOptions,
  action: (path: string) => Promise<void>,
  successMsg: string,
  errorMsg: string,
  showSuccess: (msg: string) => void,
  showError: (msg: string) => void,
): Promise<void> {
  const { save, open } = await import('@tauri-apps/plugin-dialog')
  const path = type === 'save'
    ? await save(dialogOptions as SaveDialogOptions)
    : await open(dialogOptions as OpenDialogOptions)
  if (!path) return

  try {
    await action(path as string)
    showSuccess(successMsg)
  } catch (e) {
    showError(`${errorMsg}: ${(e as Error).message}`)
  }
}

export function useImportExport() {
  const { showSuccess, showError } = useToast()
  const { t } = useI18n()

  const exportPlaylist = (playlistId: string, playlistName: string) => withFileDialog(
    'save',
    {
      defaultPath: `${playlistName}`,
      filters: [
        { name: 'M3U Playlist', extensions: ['m3u'] },
        { name: 'PLS Playlist', extensions: ['pls'] },
      ],
    },
    async (path) => {
      const ext = path.split('.').pop()?.toLowerCase()
      if (ext === 'pls') {
        await invokeCommand('export_playlist_pls', { playlistId, savePath: path })
      } else {
        await invokeCommand('export_playlist_m3u', { playlistId, savePath: path })
      }
    },
    t('toast.playlistExported'),
    t('toast.playlistExportFailed'),
    showSuccess,
    showError,
  )

  const exportBackup = () => withFileDialog(
    'save',
    { defaultPath: 'scx-music-backup', filters: [{ name: 'JSON', extensions: ['json'] }] },
    (path) => invokeCommand('export_backup', { savePath: path }),
    t('toast.backupExported'),
    t('toast.backupExportFailed'),
    showSuccess,
    showError,
  )

  async function importBackup(
    filePath: string,
    strategy: string,
  ): Promise<ImportResult | null> {
    try {
      const result = await invokeCommand<ImportResult>('import_backup', {
        filePath,
        strategy,
      })
      showSuccess(
        t('toast.backupImported', {
          songs: result.songsImported,
          playlists: result.playlistsImported,
          lyrics: result.lyricsImported,
        }),
      )
      return result
    } catch {
      showError(t('toast.backupImportFailed'))
      return null
    }
  }

  const exportSettings = () => withFileDialog(
    'save',
    { defaultPath: 'scx-music-settings', filters: [{ name: 'JSON', extensions: ['json'] }] },
    (path) => invokeCommand('export_settings', { savePath: path }),
    t('toast.settingsExported'),
    t('toast.settingsExportFailed'),
    showSuccess,
    showError,
  )

  async function importSettings() {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog')
      const selected = await open({
        multiple: false,
        filters: [{ name: 'JSON', extensions: ['json'] }],
      })
      if (!selected) return

      const count = await invokeCommand<number>('import_settings', { filePath: selected })
      showSuccess(t('toast.settingsImported', { count }))
    } catch {
      showError(t('toast.settingsImportFailed'))
    }
  }

  return {
    exportPlaylist,
    exportBackup,
    importBackup,
    exportSettings,
    importSettings,
  }
}
