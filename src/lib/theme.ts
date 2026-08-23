import { ref } from 'vue'

export type ThemePreference = 'system' | 'light' | 'dark'

const STORAGE_KEY = 'elwright.theme-preference'
const systemMedia = window.matchMedia('(prefers-color-scheme: dark)')

function readPreference(): ThemePreference {
  const saved = window.localStorage.getItem(STORAGE_KEY)
  return saved === 'light' || saved === 'dark' || saved === 'system' ? saved : 'system'
}

export const themePreference = ref<ThemePreference>(readPreference())

function resolvedTheme(preference: ThemePreference) {
  return preference === 'system' ? (systemMedia.matches ? 'dark' : 'light') : preference
}

/** 已解析的当前主题（'light' | 'dark'），跟随偏好与系统变化。 */
export const resolvedThemeRef = ref<'light' | 'dark'>(resolvedTheme(themePreference.value))

export function applyTheme(preference = themePreference.value) {
  const theme = resolvedTheme(preference)
  document.documentElement.dataset.theme = theme
  document.documentElement.style.colorScheme = theme
  resolvedThemeRef.value = theme
}

export function setThemePreference(preference: ThemePreference) {
  themePreference.value = preference
  window.localStorage.setItem(STORAGE_KEY, preference)
  applyTheme(preference)
}

export function initializeTheme() {
  applyTheme()
  systemMedia.addEventListener('change', () => {
    if (themePreference.value === 'system') applyTheme()
  })
}
