import { ref, watch } from 'vue'

export type ThemeMode = 'light' | 'dark' | 'system'

const theme = ref<ThemeMode>('system')

function applyTheme(mode: ThemeMode) {
  const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches

  if (mode === 'system') {
    document.documentElement.removeAttribute('data-theme')
  } else {
    document.documentElement.setAttribute('data-theme', mode)
  }
}

export function useTheme() {
  const stored = localStorage.getItem('theme') as ThemeMode | null
  if (stored && ['light', 'dark', 'system'].includes(stored)) {
    theme.value = stored
  }
  applyTheme(theme.value)

  watch(theme, (val) => {
    localStorage.setItem('theme', val)
    applyTheme(val)
  })

  return { theme }
}
