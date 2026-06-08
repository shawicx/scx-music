import { invokeCommand } from '../utils/errorHandler'
import { useToast } from './useToast'
import { useI18n } from './useI18n'

interface ImportResult {
  songsImported: number
  playlistsImported: number
  lyricsImported: number
}

export function useImportExport() {
  const { showSuccess, showError } = useToast()
  const { t } = useI18n()

  async function exportPlaylist(playlistId: string, playlistName: string) {
    try {
      const { save } = await import('@tauri-apps/plugin-dialog')
      const path = await save({
        defaultPath: `${playlistName}`,
        filters: [
          { name: 'M3U Playlist', extensions: ['m3u'] },
          { name: 'PLS Playlist', extensions: ['pls'] },
        ],
      })
      if (!path) return

      const ext = path.split('.').pop()?.toLowerCase()
      if (ext === 'pls') {
        await invokeCommand('export_playlist_pls', { playlistId, savePath: path })
      } else {
        await invokeCommand('export_playlist_m3u', { playlistId, savePath: path })
      }
      showSuccess(t('toast.playlistExported'))
    } catch {
      showError(t('toast.playlistExportFailed'))
    }
  }

  async function exportBackup() {
    try {
      const { save } = await import('@tauri-apps/plugin-dialog')
      const path = await save({
        defaultPath: 'scx-music-backup',
        filters: [{ name: 'JSON', extensions: ['json'] }],
      })
      if (!path) return

      await invokeCommand('export_backup', { savePath: path })
      showSuccess(t('toast.backupExported'))
    } catch {
      showError(t('toast.backupExportFailed'))
    }
  }

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

  async function exportSettings() {
    try {
      const { save } = await import('@tauri-apps/plugin-dialog')
      const path = await save({
        defaultPath: 'scx-music-settings',
        filters: [{ name: 'JSON', extensions: ['json'] }],
      })
      if (!path) return

      await invokeCommand('export_settings', { savePath: path })
      showSuccess(t('toast.settingsExported'))
    } catch {
      showError(t('toast.settingsExportFailed'))
    }
  }

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
