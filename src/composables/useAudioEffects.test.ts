import { describe, it, expect, vi, beforeEach } from 'vitest'

// mock invokeCommand：按命令名路由返回值/拒绝
const invokeCommandMock = vi.fn<(cmd: string, args?: Record<string, unknown>) => Promise<unknown>>()
vi.mock('../utils/errorHandler', () => ({
  invokeCommand: (cmd: string, args?: Record<string, unknown>) => invokeCommandMock(cmd, args),
}))

import { useAudioEffects } from './useAudioEffects'
import type { EqParams, EqPresetDef } from '../types'

const PRESETS: EqPresetDef[] = [
  { id: 'flat', preampDb: 0, gainsDb: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
  { id: 'rock', preampDb: -3, gainsDb: [5, 4, 3, 1, -1, -1, 1, 3, 5, 5] },
]

function eqParams(overrides: Partial<EqParams> = {}): EqParams {
  return {
    presetId: 'rock',
    enabled: true,
    preampDb: -3,
    gainsDb: [5, 4, 3, 1, -1, -1, 1, 3, 5, 5],
    ...overrides,
  }
}

/** 按命令名捕获调用参数 */
function callsOf(cmd: string) {
  return invokeCommandMock.mock.calls.filter(([c]) => c === cmd)
}

beforeEach(() => {
  invokeCommandMock.mockReset()
  invokeCommandMock.mockImplementation((cmd) => {
    if (cmd === 'player_list_eq_presets') return Promise.resolve(PRESETS)
    if (cmd === 'player_set_eq_preset') return Promise.resolve(eqParams())
    if (cmd === 'player_set_eq_enabled') return Promise.resolve(eqParams())
    return Promise.resolve(undefined)
  })
})

describe('useAudioEffects.init', () => {
  it('加载预设列表并应用持久化的预设与开关', async () => {
    const { presets, currentPresetId, enabled, init } = useAudioEffects()

    await init({ eq_preset: 'rock', eq_enabled: 'true' })

    expect(presets.value).toEqual(PRESETS)
    expect(currentPresetId.value).toBe('rock')
    expect(enabled.value).toBe(true)
    expect(callsOf('player_set_eq_preset')).toHaveLength(1)
    expect(callsOf('player_set_eq_preset')[0][1]).toEqual({ presetId: 'rock' })
    expect(callsOf('player_set_eq_enabled')).toHaveLength(1)
    expect(callsOf('player_set_eq_enabled')[0][1]).toEqual({ enabled: true })
  })

  it('无持久化设置时保持默认 flat/disabled 且不发起应用调用', async () => {
    const { presets, currentPresetId, enabled, init } = useAudioEffects()

    await init({})

    expect(presets.value).toEqual(PRESETS)
    expect(currentPresetId.value).toBe('flat')
    expect(enabled.value).toBe(false)
    expect(callsOf('player_set_eq_preset')).toHaveLength(0)
    expect(callsOf('player_set_eq_enabled')).toHaveLength(0)
  })

  it('持久化了坏预设 id 时静默跳过不抛错', async () => {
    invokeCommandMock.mockImplementation((cmd) => {
      if (cmd === 'player_list_eq_presets') return Promise.resolve(PRESETS)
      if (cmd === 'player_set_eq_preset') {
        return Promise.reject(Object.assign(new Error('bad'), { type: 'InvalidArgument' }))
      }
      if (cmd === 'player_set_eq_enabled') {
        // 后端拒绝了坏预设后仍是 flat，set_eq_enabled 如实返回实际状态
        return Promise.resolve(eqParams({ presetId: 'flat', preampDb: 0, gainsDb: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }))
      }
      return Promise.resolve(undefined)
    })
    const { currentPresetId, enabled, init } = useAudioEffects()

    await expect(init({ eq_preset: 'ghost', eq_enabled: 'true' })).resolves.toBeUndefined()

    expect(currentPresetId.value).toBe('flat')
    // 预设失败不应阻断 enabled 恢复
    expect(callsOf('player_set_eq_enabled')).toHaveLength(1)
    expect(enabled.value).toBe(true)
  })
})

describe('useAudioEffects.switchPreset', () => {
  it('调用 IPC 切换预设并持久化 eq_preset', async () => {
    const { currentPresetId, switchPreset } = useAudioEffects()

    await switchPreset('rock')

    expect(callsOf('player_set_eq_preset')).toHaveLength(1)
    expect(callsOf('player_set_eq_preset')[0][1]).toEqual({ presetId: 'rock' })
    expect(callsOf('set_setting')).toHaveLength(1)
    expect(callsOf('set_setting')[0][1]).toEqual({ key: 'eq_preset', value: 'rock' })
    expect(currentPresetId.value).toBe('rock')
  })

  it('IPC 失败时状态回滚且不持久化', async () => {
    invokeCommandMock.mockRejectedValue(new Error('boom'))
    const { currentPresetId, switchPreset } = useAudioEffects()

    await switchPreset('rock')

    expect(currentPresetId.value).toBe('flat')
    expect(callsOf('set_setting')).toHaveLength(0)
  })
})

describe('useAudioEffects.setEnabled', () => {
  it('调用 IPC 设置开关并持久化 eq_enabled', async () => {
    const { enabled, setEnabled } = useAudioEffects()

    await setEnabled(true)

    expect(callsOf('player_set_eq_enabled')).toHaveLength(1)
    expect(callsOf('player_set_eq_enabled')[0][1]).toEqual({ enabled: true })
    expect(callsOf('set_setting')).toHaveLength(1)
    expect(callsOf('set_setting')[0][1]).toEqual({ key: 'eq_enabled', value: 'true' })
    expect(enabled.value).toBe(true)
  })

  it('IPC 失败时状态回滚且不持久化', async () => {
    invokeCommandMock.mockRejectedValue(new Error('boom'))
    const { enabled, setEnabled } = useAudioEffects()

    await setEnabled(true)

    expect(enabled.value).toBe(false)
    expect(callsOf('set_setting')).toHaveLength(0)
  })
})
