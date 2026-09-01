// i18n 基建（设置中心 ADR-002）：轻量 t() + 双语字典，零依赖。
// zh-CN 为源语言；缺 key 回退 zh-CN，再回退 key 本身。
// 迁移策略：设置中心壳层为试点，其余视图按需增量接入（见 ADR-002 第 3 条）。

import { ref } from 'vue'

export type Locale = 'zh-CN' | 'en'

export const LOCALE_OPTIONS: { value: Locale; label: string }[] = [
  { value: 'zh-CN', label: '中文' },
  { value: 'en', label: 'English' },
]

export const locale = ref<Locale>('zh-CN')

export function setLocale(value: Locale): void {
  locale.value = value
}

const zh: Record<string, string> = {
  // ---- 壳层 ----
  'settings.title': '设置',
  'settings.close': '关闭设置',
  'settings.savedLocally': '设置保存在本机',
  'settings.section.general': '常规',
  'settings.section.appearance': '外观',
  'settings.section.model': '模型设置',
  'settings.section.terminal': '终端',
  // ---- 消息中继 ----
  'settings.section.messaging': '消息中继',
  'settings.messaging.desc': '人与人消息会话的转发服务。自托管后填入地址（ws:// 或 wss://）；留空时消息仅存本机，不发起网络连接。',
  'settings.messaging.url': '中继地址',
  'settings.messaging.urlPlaceholder': 'wss://relay.example.com:9443',
  'settings.messaging.save': '保存',
  'settings.messaging.test': '测试连接',
  'settings.messaging.saved': '已保存',
  'settings.messaging.testing': '测试中…',
  'settings.messaging.notConfigured': '未配置中继（消息仅本机保存）',
  // ---- 常规 ----
  'settings.general.desc': '应用行为偏好，保存在本机。',
  'settings.general.startupView': '启动视图',
  'settings.general.autoUpdate': '启动时自动检查更新',
  'settings.general.language': '界面语言',
  // ---- 外观 ----
  'settings.appearance.desc': '选择 Elwright 的界面主题。',
  'settings.appearance.theme.system': '跟随系统',
  'settings.appearance.theme.systemNote': '根据操作系统自动切换',
  'settings.appearance.theme.light': '浅色',
  'settings.appearance.theme.lightNote': '适合明亮环境',
  'settings.appearance.theme.dark': '深色',
  'settings.appearance.theme.darkNote': '适合低光环境',
  'settings.appearance.density': '界面密度',
  'settings.appearance.density.comfortable': '舒适',
  'settings.appearance.density.compact': '紧凑',
  'settings.appearance.scale': '界面缩放',
  // ---- 模型 ----
  'settings.model.desc': '配置 OpenAI 兼容模型端点，桌面应用与 CLI 共用。',
  // ---- 终端 ----
  'settings.terminal.desc': '字号即时生效；滚动历史对新输出生效。主题已随界面主题自动切换。',
  'settings.terminal.fontFamily': '终端字体',
  'settings.terminal.fontSize': '终端字号',
  'settings.terminal.scrollback': '滚动历史',
  'settings.terminal.scrollbackUnit': '行',
}

const en: Record<string, string> = {
  'settings.title': 'Settings',
  'settings.close': 'Close settings',
  'settings.savedLocally': 'Settings are stored locally',
  'settings.section.general': 'General',
  'settings.section.appearance': 'Appearance',
  'settings.section.model': 'Model',
  'settings.section.terminal': 'Terminal',
  'settings.section.messaging': 'Messaging relay',
  'settings.messaging.desc': 'Relay service for people chat. Self-host one and paste its address (ws:// or wss://); leave empty to keep messages local with no network.',
  'settings.messaging.url': 'Relay URL',
  'settings.messaging.urlPlaceholder': 'wss://relay.example.com:9443',
  'settings.messaging.save': 'Save',
  'settings.messaging.test': 'Test connection',
  'settings.messaging.saved': 'Saved',
  'settings.messaging.testing': 'Testing…',
  'settings.messaging.notConfigured': 'Relay not configured (messages stay local)',
  'settings.general.desc': 'App behavior preferences, stored locally.',
  'settings.general.startupView': 'Startup view',
  'settings.general.autoUpdate': 'Check for updates on startup',
  'settings.general.language': 'Interface language',
  'settings.appearance.desc': 'Choose the Elwright interface theme.',
  'settings.appearance.theme.system': 'Follow system',
  'settings.appearance.theme.systemNote': 'Switch automatically with the OS',
  'settings.appearance.theme.light': 'Light',
  'settings.appearance.theme.lightNote': 'For bright environments',
  'settings.appearance.theme.dark': 'Dark',
  'settings.appearance.theme.darkNote': 'For low-light environments',
  'settings.appearance.density': 'Density',
  'settings.appearance.density.comfortable': 'Comfortable',
  'settings.appearance.density.compact': 'Compact',
  'settings.appearance.scale': 'UI scale',
  'settings.model.desc': 'Configure an OpenAI-compatible endpoint, shared by desktop and CLI.',
  'settings.terminal.desc':
    'Font size applies immediately; scrollback applies to new output. Theme follows the interface theme.',
  'settings.terminal.fontFamily': 'Terminal font',
  'settings.terminal.fontSize': 'Terminal font size',
  'settings.terminal.scrollback': 'Scrollback',
  'settings.terminal.scrollbackUnit': 'lines',
}

const dicts: Record<Locale, Record<string, string>> = { 'zh-CN': zh, en }

/** 翻译：当前语言 → zh-CN 回退 → key 本身。 */
export function t(key: string): string {
  return dicts[locale.value][key] ?? dicts['zh-CN'][key] ?? key
}

/** 测试与 CI 守卫用：两种语言的键集是否完全一致。 */
export function dictKeysInSync(): boolean {
  const a = Object.keys(zh).sort()
  const b = Object.keys(en).sort()
  return a.length === b.length && a.every((k, i) => k === b[i])
}
