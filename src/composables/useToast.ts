import { ref } from 'vue'

export interface ToastOptions {
  message: string
  type?: 'success' | 'error' | 'warning' | 'info'
  duration?: number
}

const toastMessage = ref('')
const toastVisible = ref(false)
const toastColor = ref<'success' | 'error' | 'warning' | 'info'>('info')
let toastTimer: ReturnType<typeof setTimeout> | null = null

export function useToast() {
  function showToast(options: string | ToastOptions) {
    const config = typeof options === 'string' ? { message: options, type: 'info' as const } : options

    toastMessage.value = config.message
    toastColor.value = config.type || 'info'
    toastVisible.value = true

    if (toastTimer) clearTimeout(toastTimer)

    const duration = config.duration ?? 3000
    toastTimer = setTimeout(() => {
      toastVisible.value = false
    }, duration)
  }

  function showSuccess(message: string, duration?: number) {
    showToast({ message, type: 'success', duration })
  }

  function showError(message: string, duration?: number) {
    showToast({ message, type: 'error', duration })
  }

  function showWarning(message: string, duration?: number) {
    showToast({ message, type: 'warning', duration })
  }

  function showInfo(message: string, duration?: number) {
    showToast({ message, type: 'info', duration })
  }

  function hideToast() {
    toastVisible.value = false
    if (toastTimer) {
      clearTimeout(toastTimer)
      toastTimer = null
    }
  }

  return {
    toastMessage,
    toastVisible,
    toastColor,
    showToast,
    showSuccess,
    showError,
    showWarning,
    showInfo,
    hideToast,
  }
}