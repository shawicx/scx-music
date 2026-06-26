import { shallowRef, ref } from 'vue'
import { invoke, Channel } from '@tauri-apps/api/core'

export const NUM_BINS = 64

export function useAudioAnalyzer() {
  const frequencyData = shallowRef<Uint8Array>(new Uint8Array(NUM_BINS))
  const isActive = ref(false)
  let channel: Channel<number[]> | null = null

  async function start() {
    if (isActive.value) return

    // Channel 点对点接收频谱数据;channel 随组件卸载/webview 销毁自动断开,
    // 后端 send 失败即停线程,无需前端手动调用 analyzer_stop 配对。
    channel = new Channel<number[]>()
    channel.onmessage = (bins) => {
      const buf = frequencyData.value
      for (let i = 0; i < Math.min(bins.length, NUM_BINS); i++) {
        buf[i] = bins[i]
      }
    }

    await invoke('analyzer_start', { onData: channel })
    isActive.value = true
  }

  async function stop() {
    if (!isActive.value) return

    // 主动停止:断开 channel 通知后端退出线程。
    // 即便前端忘记调用,webview 销毁时 channel 也会失效,后端 send 报错自动退出。
    if (channel) {
      await invoke('analyzer_stop').catch(() => {})
      channel = null
    }

    isActive.value = false
    frequencyData.value = new Uint8Array(NUM_BINS)
  }

  return { frequencyData, isActive, start, stop }
}
