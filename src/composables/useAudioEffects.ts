import { ref } from 'vue'
import { invokeCommand } from '../utils/errorHandler'
import { useToast } from './useToast'
import type { EqParams, EqPresetDef } from '../types'

/**
 * 音效（EQ 预设）管理
 *
 * - `init(settings?)`：加载预设列表并应用持久化的 eq_preset/eq_enabled（启动恢复，
 *   坏值静默跳过不阻塞启动）；未传 settings 时自行拉取 get_all_settings
 * - `switchPreset(id)`：IPC 实时生效（播放中平滑切换）+ set_setting 持久化
 * - `setEnabled(v)`：总开关（关闭 = 直通，预设保留）
 *
 * 失败处理：IPC 失败时 toast 提示且状态不变更（后端为权威源，成功才落地）。
 */
export function useAudioEffects() {
  const toast = useToast()
  const presets = ref<EqPresetDef[]>([])
  const currentPresetId = ref('flat')
  const enabled = ref(false)

  function applyParams(params: EqParams) {
    currentPresetId.value = params.presetId
    enabled.value = params.enabled
  }

  async function init(settings?: Record<string, string>) {
    try {
      presets.value = await invokeCommand<EqPresetDef[]>('player_list_eq_presets')
      const map = settings ?? (await invokeCommand<Record<string, string>>('get_all_settings'))
      const savedPreset = map['eq_preset']
      if (savedPreset && savedPreset !== 'flat') {
        try {
          applyParams(await invokeCommand<EqParams>('player_set_eq_preset', { presetId: savedPreset }))
        } catch {
          // 坏预设 id（如后端预设表变更）静默跳过，不阻塞启动
        }
      }
      if (map['eq_enabled'] === 'true') {
        try {
          applyParams(await invokeCommand<EqParams>('player_set_eq_enabled', { enabled: true }))
        } catch {
          // 同上
        }
      }
    } catch (e) {
      console.warn('[audio-effects] init failed:', e)
    }
  }

  async function switchPreset(id: string) {
    try {
      applyParams(await invokeCommand<EqParams>('player_set_eq_preset', { presetId: id }))
      await invokeCommand('set_setting', { key: 'eq_preset', value: id })
    } catch (e) {
      toast.showToast(String(e))
    }
  }

  async function setEnabled(value: boolean) {
    try {
      applyParams(await invokeCommand<EqParams>('player_set_eq_enabled', { enabled: value }))
      await invokeCommand('set_setting', { key: 'eq_enabled', value: String(value) })
    } catch (e) {
      toast.showToast(String(e))
    }
  }

  return { presets, currentPresetId, enabled, init, switchPreset, setEnabled }
}
