import { createI18n } from 'vue-i18n'
import zhCN from './zh-CN.json'
import en from './en.json'

const messages = {
  'zh-CN': zhCN,
  en
}

const supportedLocales = Object.keys(messages)

export function resolveSystemLocale(): string {
  const lang = navigator.language
  const exact = supportedLocales.find((l) => l === lang)
  if (exact) return exact
  const prefix = supportedLocales.find((l) => lang.startsWith(l.split('-')[0]!))
  if (prefix) return prefix
  return 'en'
}

export const i18n = createI18n({
  legacy: false,
  locale: resolveSystemLocale(),
  fallbackLocale: 'en',
  messages
})
