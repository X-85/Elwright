import { ref, watch } from 'vue'
import { setLocale, type Locale } from './i18n'

/**
 * 设置中心后续阶段的本地偏好（常规/外观/终端）。
 * 与 theme.ts 同源思路：localStorage 持久化 + reactive + 启动时应用。
 * 浏览器预览与桌面壳行为一致（纯前端偏好，不涉 IPC）。
 */

export type StartupView = 'last' | 'toolbox' | 'workbench' | 'chat' | 'people' | 'workspace' | 'code' | 'mindmap'
export type Density = 'comfortable' | 'compact'
export type UiScale = 90 | 100 | 110 | 125

export interface Preferences {
  /** 启动时进入的视图：last = 记住上次；或固定某个视图。 */
  startupView: StartupView
  /** 启动时自动检查更新。 */
  autoUpdateCheck: boolean
  /** 界面密度。 */
  density: Density
  /** 界面缩放百分比。 */
  uiScale: UiScale
  /** 终端字体（CSS font-family 值）。 */
  terminalFontFamily: string
  /** 终端字号。 */
  terminalFontSize: number
  /** 终端滚动历史行数。 */
  terminalScrollback: number
  /** 界面语言（ADR-002 i18n 基建；存量视图为增量迁移，设置中心壳层已接入）。 */
  language: Locale
}

export const TERMINAL_FONT_OPTIONS: { value: string; label: string }[] = [
  { value: 'Menlo, Consolas, "Liberation Mono", monospace', label: '默认（Menlo / Consolas）' },
  { value: '"SF Mono", Menlo, monospace', label: 'SF Mono' },
  { value: '"JetBrains Mono", Menlo, Consolas, monospace', label: 'JetBrains Mono' },
  { value: '"Cascadia Code", Consolas, Menlo, monospace', label: 'Cascadia Code' },
]

export const UI_SCALE_OPTIONS: UiScale[] = [90, 100, 110, 125]
export const TERMINAL_FONT_SIZE_OPTIONS = [11, 12, 13, 14, 16, 18]
export const TERMINAL_SCROLLBACK_OPTIONS = [1000, 5000, 10000, 50000]

export const STARTUP_VIEW_OPTIONS: { value: StartupView; labelKey: string }[] = [
  { value: 'last', labelKey: 'startup.last' },
  { value: 'toolbox', labelKey: 'startup.toolbox' },
  { value: 'workbench', labelKey: 'startup.workbench' },
  { value: 'chat', labelKey: 'startup.chat' },
  { value: 'people', labelKey: 'startup.people' },
  { value: 'workspace', labelKey: 'startup.workspace' },
  { value: 'code', labelKey: 'startup.code' },
  { value: 'mindmap', labelKey: 'startup.mindmap' },
]

const DEFAULTS: Preferences = {
  startupView: 'last',
  autoUpdateCheck: false,
  density: 'comfortable',
  uiScale: 100,
  terminalFontFamily: TERMINAL_FONT_OPTIONS[0].value,
  terminalFontSize: 13,
  terminalScrollback: 10000,
  language: 'zh-CN',
}

const STORAGE_KEY = 'elwright.preferences'
const LAST_VIEW_KEY = 'elwright.last-view'

/** 把任意来源（localStorage JSON）的值合并到默认值上，逐字段校验类型。 */
export function mergePreferences(raw: unknown): Preferences {
  const out: Preferences = { ...DEFAULTS }
  if (typeof raw !== 'object' || raw === null) return out
  const r = raw as Record<string, unknown>
  if (STARTUP_VIEW_OPTIONS.some((o) => o.value === r.startupView)) {
    out.startupView = r.startupView as StartupView
  }
  if (typeof r.autoUpdateCheck === 'boolean') out.autoUpdateCheck = r.autoUpdateCheck
  if (r.density === 'comfortable' || r.density === 'compact') out.density = r.density
  if (UI_SCALE_OPTIONS.includes(r.uiScale as UiScale)) out.uiScale = r.uiScale as UiScale
  if (typeof r.terminalFontFamily === 'string' && r.terminalFontFamily.trim()) {
    out.terminalFontFamily = r.terminalFontFamily
  }
  if (TERMINAL_FONT_SIZE_OPTIONS.includes(r.terminalFontSize as number)) {
    out.terminalFontSize = r.terminalFontSize as number
  }
  if (TERMINAL_SCROLLBACK_OPTIONS.includes(r.terminalScrollback as number)) {
    out.terminalScrollback = r.terminalScrollback as number
  }
  if (r.language === 'zh-CN' || r.language === 'en') out.language = r.language
  return out
}

function read(): Preferences {
  try {
    return mergePreferences(JSON.parse(window.localStorage.getItem(STORAGE_KEY) ?? 'null'))
  } catch {
    return { ...DEFAULTS }
  }
}

export const preferences = ref<Preferences>(read())

// 语言偏好与 i18n 运行时同步（初始化 + 每次变更）
setLocale(preferences.value.language)

export function updatePreferences(patch: Partial<Preferences>) {
  preferences.value = { ...preferences.value, ...patch }
  if (patch.language) setLocale(patch.language)
}

function applyPreferences(p: Preferences) {
  document.documentElement.classList.toggle('density-compact', p.density === 'compact')
  document.documentElement.style.setProperty('zoom', p.uiScale === 100 ? '' : String(p.uiScale / 100))
}

/** 应用当前偏好并跟随变化（App 挂载时调用一次）。 */
export function initializePreferences() {
  applyPreferences(preferences.value)
  watch(
    preferences,
    (p) => {
      applyPreferences(p)
      try {
        window.localStorage.setItem(STORAGE_KEY, JSON.stringify(p))
      } catch { /* 持久化失败不阻塞运行 */ }
    },
    { deep: true },
  )
}

const VALID_VIEWS = STARTUP_VIEW_OPTIONS.map((o) => o.value).filter((v) => v !== 'last')

/** 记住上次视图（launch 恢复用）。 */
export function saveLastView(view: string) {
  if (VALID_VIEWS.includes(view as StartupView)) {
    try {
      window.localStorage.setItem(LAST_VIEW_KEY, view)
    } catch { /* ignore */ }
  }
}

/** 解析启动视图偏好：last → 上次视图（无记录则工具箱），固定值直接返回。 */
export function resolveStartupView(preference: StartupView): Exclude<StartupView, 'last'> {
  if (preference !== 'last') return preference
  const saved = window.localStorage.getItem(LAST_VIEW_KEY)
  return VALID_VIEWS.includes(saved as StartupView) ? (saved as Exclude<StartupView, 'last'>) : 'toolbox'
}
