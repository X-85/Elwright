// jsdom 无 matchMedia 实现，而 theme.ts 在模块加载时（顶层）就调用它——
// stub 必须先于模块加载生效，因此用动态 import 放在 stub 之后。
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mediaListeners: Array<(e: { matches: boolean }) => void> = []
let mediaDark = false

vi.stubGlobal('matchMedia', (_query: string) => ({
  // getter 而非快照：theme.ts 模块加载时只调一次 matchMedia 并持有该对象，
  // 之后通过 systemMedia.matches 读取——必须是活值才能模拟系统偏好切换
  get matches() {
    return mediaDark
  },
  addEventListener: (_t: string, cb: (e: { matches: boolean }) => void) => mediaListeners.push(cb),
  removeEventListener: () => {},
}))

const { applyTheme, initializeTheme, resolvedThemeRef, setThemePreference, themePreference } =
  await import('../theme')

describe('theme · 三态偏好', () => {
  beforeEach(() => {
    localStorage.clear()
    mediaDark = false
    mediaListeners.length = 0
  })

  it('默认偏好为 system', () => {
    expect(themePreference.value).toBe('system')
  })

  it('light/dark 偏好直接生效到 data-theme', () => {
    setThemePreference('dark')
    expect(document.documentElement.dataset.theme).toBe('dark')
    setThemePreference('light')
    expect(document.documentElement.dataset.theme).toBe('light')
  })

  it('system 模式解析跟随系统偏好（dark → light）', () => {
    // initializeTheme 注册「系统偏好变化 → 重新 applyTheme」监听（App 启动接线）
    initializeTheme()
    mediaDark = true
    setThemePreference('system')
    expect(resolvedThemeRef.value).toBe('dark')

    // 模拟系统切换到浅色：触发监听器
    mediaDark = false
    mediaListeners.forEach((cb) => cb({ matches: false }))
    expect(resolvedThemeRef.value).toBe('light')
    expect(document.documentElement.dataset.theme).toBe('light')
  })

  it('偏好持久化到 localStorage', () => {
    setThemePreference('dark')
    expect(localStorage.getItem('elwright.theme-preference')).toBe('dark')
  })

  it('applyTheme 同步 colorScheme（原生控件跟随）', () => {
    setThemePreference('dark')
    expect(document.documentElement.style.colorScheme).toBe('dark')
  })
})
