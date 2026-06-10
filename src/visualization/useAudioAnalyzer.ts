import { shallowRef, ref } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'

export const NUM_BINS = 64

export function useAudioAnalyzer() {
  const frequencyData = shallowRef<Uint8Array>(new Uint8Array(NUM_BINS))
  const isActive = ref(false)
  let unlisten: UnlistenFn | null = null

  async function start() {
    if (isActive.value) return

    unlisten = await listen<number[]>('audio:spectrum', (e) => {
      const buf = frequencyData.value
      const payload = e.payload
      for (let i = 0; i < Math.min(payload.length, NUM_BINS); i++) {
        buf[i] = payload[i]
      }
    })

    await invoke('analyzer_start')
    isActive.value = true
  }

  async function stop() {
    if (!isActive.value) return

    if (unlisten) {
      unlisten()
      unlisten = null
    }

    await invoke('analyzer_stop')
    isActive.value = false
    frequencyData.value = new Uint8Array(NUM_BINS)
  }

  return { frequencyData, isActive, start, stop }
}
