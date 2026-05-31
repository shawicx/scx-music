import { computed } from 'vue'
import { useI18n as useVueI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'

const SUPPORTED_LOCALES = ['zh-CN', 'en'] as const
type SupportedLocale = (typeof SUPPORTED_LOCALES)[number]
type LocaleSetting = SupportedLocale | 'system'

function normalizeLocale(raw: string): SupportedLocale {
  if (raw.startsWith('zh')) return 'zh-CN'
  if (raw.startsWith('en')) return 'en'
  return 'zh-CN'
}

export type { LocaleSetting, SupportedLocale }

export function useI18n() {
  const { t, locale } = useVueI18n()

  const currentLocale = computed(() => locale.value as SupportedLocale)

  async function initLocale() {
    try {
      const saved = await invoke<string | null>('get_setting', { key: 'language' })

      let resolved: SupportedLocale
      if (saved && saved !== 'system') {
        resolved = normalizeLocale(saved)
      } else {
        const sysLocale = await invoke<string>('get_system_locale')
        resolved = normalizeLocale(sysLocale)
      }

      locale.value = resolved
    } catch {
      locale.value = 'zh-CN'
    }
  }

  async function setLocale(setting: LocaleSetting) {
    try {
      await invoke('set_setting', { key: 'language', value: setting })

      if (setting === 'system') {
        const sysLocale = await invoke<string>('get_system_locale')
        locale.value = normalizeLocale(sysLocale)
      } else {
        locale.value = setting
      }
    } catch {
      // keep current locale
    }
  }

  async function getLocaleSetting(): Promise<LocaleSetting> {
    try {
      const saved = await invoke<string | null>('get_setting', { key: 'language' })
      if (saved === 'system' || saved === 'zh-CN' || saved === 'en') return saved
      return 'system'
    } catch {
      return 'system'
    }
  }

  return { t, locale, currentLocale, initLocale, setLocale, getLocaleSetting }
}
