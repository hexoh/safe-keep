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

// 日期 / 数字格式集中维护，页面不再自行拼接格式字符串（feature 3.2.10）。
const datetimeFormats = {
  'zh-CN': {
    short: { year: 'numeric', month: '2-digit', day: '2-digit' },
    long: {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      hour12: false
    }
  },
  en: {
    short: { year: 'numeric', month: 'short', day: 'numeric' },
    long: {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: 'numeric',
      minute: '2-digit',
      hour12: true
    }
  }
} as const

const numberFormats = {
  'zh-CN': {
    decimal: { style: 'decimal', maximumFractionDigits: 2, useGrouping: true },
    integer: { style: 'decimal', maximumFractionDigits: 0, useGrouping: true }
  },
  en: {
    decimal: { style: 'decimal', maximumFractionDigits: 2, useGrouping: true },
    integer: { style: 'decimal', maximumFractionDigits: 0, useGrouping: true }
  }
} as const

export const i18n = createI18n({
  legacy: false,
  locale: resolveSystemLocale(),
  fallbackLocale: 'en',
  messages,
  datetimeFormats,
  numberFormats
})
