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
  // ---- 枚举标签（增量迁移 ADR-002 第 3 条）----
  'startup.last': '记住上次视图',
  'startup.toolbox': '能力工具箱',
  'startup.workbench': '工作台',
  'startup.chat': 'AI 对话',
  'startup.people': '消息会话',
  'startup.workspace': '资源与课题',
  'startup.code': '代码浏览器',
  'startup.mindmap': '脑图',
  // ---- 模型设置（增量迁移）----
  'llm.title': '⚙ 模型设置',
  'llm.hint': 'OpenAI 兼容端点（云端 API 或本地 Ollama / llama.cpp）。保存在',
  'llm.hintTail': '，桌面应用与 CLI（ew config）共用。',
  'llm.profile': '档案',
  'llm.flatOption': '（flat 字段，未指定档案）',
  'llm.add': '新建档案',
  'llm.source': '来源：',
  'llm.stored': '已存：',
  'llm.baseUrlPlaceholder': '如 https://api.xxx.com/v1 或 http://localhost:11434/v1',
  'llm.modelPlaceholder': '如 gpt-4o-mini / qwen3:8b',
  'llm.apiKeyPlaceholder': '留空 = 不修改已保存的 key',
  'llm.test': '测试连接',
  'llm.testing': '测试中…',
  'llm.save': '保存',
  'llm.saving': '保存中…',
  'llm.saved': '已保存（写入用户层，桌面与 CLI 共用）',
  'llm.configured': '已配置档案',
  'llm.delete': '删除',
  'llm.addTitle': '新建档案',
  'llm.addHint': '档案名将作为本次表单值（base_url / api_key / model）的命名副本保存。仅小写字母/数字/-/_，长度 1-32。',
  'llm.namePlaceholder': '如 local-ollama / work / default',
  'llm.cancel': '取消',
  'llm.saveProfile': '保存档案',
  'llm.switchedTo': '已切换到档案：{name}',
  'llm.usingFlat': '当前使用 flat 字段（未指定档案）',
  'llm.created': '已新建档案：{name}',
  'llm.deleted': '已删除档案：{name}',
  'llm.confirmDelete': '确认删除档案 "{name}"？{activeNote}',
  'llm.confirmDeleteActive': '当前正在使用，将自动回退 flat 字段。',
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
  'startup.last': 'Remember last view',
  'startup.toolbox': 'Capability toolbox',
  'startup.workbench': 'Workbench',
  'startup.chat': 'AI chat',
  'startup.people': 'People chat',
  'startup.workspace': 'Resources & topics',
  'startup.code': 'Code browser',
  'startup.mindmap': 'Mind map',
  'llm.title': '⚙ Model settings',
  'llm.hint': 'OpenAI-compatible endpoint (cloud API or local Ollama / llama.cpp). Saved to',
  'llm.hintTail': ', shared by the desktop app and CLI (ew config).',
  'llm.profile': 'Profile',
  'llm.flatOption': '(flat fields, no profile selected)',
  'llm.add': 'New profile',
  'llm.source': 'Source: ',
  'llm.stored': 'Stored: ',
  'llm.baseUrlPlaceholder': 'e.g. https://api.xxx.com/v1 or http://localhost:11434/v1',
  'llm.modelPlaceholder': 'e.g. gpt-4o-mini / qwen3:8b',
  'llm.apiKeyPlaceholder': 'Leave empty to keep the saved key',
  'llm.test': 'Test connection',
  'llm.testing': 'Testing…',
  'llm.save': 'Save',
  'llm.saving': 'Saving…',
  'llm.saved': 'Saved (written to user config, shared by desktop and CLI)',
  'llm.configured': 'Configured profiles',
  'llm.delete': 'Delete',
  'llm.addTitle': 'New profile',
  'llm.addHint': 'The profile name saves the current form values (base_url / api_key / model) as a named copy. Lowercase letters/digits/-/_ only, length 1-32.',
  'llm.namePlaceholder': 'e.g. local-ollama / work / default',
  'llm.cancel': 'Cancel',
  'llm.saveProfile': 'Save profile',
  'llm.switchedTo': 'Switched to profile: {name}',
  'llm.usingFlat': 'Using flat fields (no profile selected)',
  'llm.created': 'Profile created: {name}',
  'llm.deleted': 'Profile deleted: {name}',
  'llm.confirmDelete': 'Delete profile "{name}"? {activeNote}',
  'llm.confirmDeleteActive': 'It is currently active and will fall back to flat fields.',
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
