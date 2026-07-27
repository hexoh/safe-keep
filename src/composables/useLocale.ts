import { ref, watch } from 'vue'
import { i18n, resolveSystemLocale } from '@/locales'

export type Language = 'system' | 'zh-CN' | 'en'

const language = ref<Language>('system')
let initialized = false

function applyLanguage(lang: Language) {
  const locale = lang === 'system' ? resolveSystemLocale() : lang
  i18n.global.locale.value = locale as any
}

function onLanguageChange() {
  if (language.value === 'system') {
    applyLanguage('system')
  }
}

export function useLocale() {
  if (!initialized) {
    initialized = true
    const stored = localStorage.getItem('language') as Language | null
    if (stored && ['system', 'zh-CN', 'en'].includes(stored)) {
      language.value = stored
    }
    applyLanguage(language.value)

    window.addEventListener('languagechange', onLanguageChange)

    watch(language, (val) => {
      localStorage.setItem('language', val)
      applyLanguage(val)
    })
  }

  return { language }
}
