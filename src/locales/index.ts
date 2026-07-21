import { createI18n } from 'vue-i18n'
import zhCN from './zh-CN.json'
import en from './en.json'

const defaultLocale = (() => {
  const lang = navigator.language
  if (lang.startsWith('zh')) return 'zh-CN'
  return 'en'
})()

export const i18n = createI18n({
  legacy: false,
  locale: defaultLocale,
  fallbackLocale: 'en',
  messages: {
    'zh-CN': zhCN,
    en
  }
})
