import { ref, watch } from 'vue'

export type ThemeMode = 'light' | 'dark' | 'system'

const theme = ref<ThemeMode>('system')
let initialized = false
let mediaQuery: MediaQueryList | null = null

function applySystemTheme() {
  const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
  if (prefersDark) {
    document.documentElement.removeAttribute('data-theme')
  } else {
    document.documentElement.setAttribute('data-theme', 'light')
  }
}

function onSystemThemeChange() {
  if (theme.value === 'system') {
    applySystemTheme()
  }
}

function applyTheme(mode: ThemeMode) {
  if (mode === 'system') {
    applySystemTheme()
    if (!mediaQuery) {
      mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
      mediaQuery.addEventListener('change', onSystemThemeChange)
    }
  } else {
    if (mediaQuery) {
      mediaQuery.removeEventListener('change', onSystemThemeChange)
      mediaQuery = null
    }
    document.documentElement.setAttribute('data-theme', mode)
  }
}

export function useTheme() {
  if (!initialized) {
    initialized = true
    const stored = localStorage.getItem('theme') as ThemeMode | null
    if (stored && ['light', 'dark', 'system'].includes(stored)) {
      theme.value = stored
    }
    applyTheme(theme.value)

    watch(theme, (val) => {
      localStorage.setItem('theme', val)
      applyTheme(val)
    })
  }

  return { theme }
}
