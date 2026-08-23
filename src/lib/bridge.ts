/// 桌面壳与核心的边界抽象：UI 只依赖 Bridge 接口，不关心底下是
/// 浏览器预览（dev 中间件 /api/*）还是 Tauri IPC。

export interface Capability {
  id: string
  name: string
  type: 'script' | 'knowledge' | 'skill' | string
  category?: string
  entry?: string
  doc?: string
  offline?: boolean
  prompt?: string
  degradeDoc?: string
  /** builtin = 内置注册表；custom = 用户叠加层（~/.elwright/）导入 */
  origin?: 'builtin' | 'custom'
}

export interface ViewResult {
  ok: boolean
  content: string
  path?: string
}

export interface RunResult {
  ok: boolean
  output: string
}

export interface InvokeResult {
  source: 'llm' | 'degraded'
  content: string
  note?: string
}

export interface UpdateInfo {
  current: string
  latest: string
  updateAvailable: boolean
  releaseUrl: string
}

export interface ImportResult {
  ok: boolean
  message: string
  /** id 冲突（需确认覆盖后带 force 重试） */
  conflict?: boolean
}

/** LLM 配置生效视图（api_key 打码，不回传明文） */
export interface LlmConfigInfo {
  baseUrl: string
  model: string
  apiKeyMasked: string
  /** 每字段来源标签：[baseUrl, apiKey, model] */
  source: [string, string, string]
  userConfigPath?: string
}

/** 多轮对话消息（前端只传 user/assistant；system 由 Rust 侧固定前置） */
export interface ChatMessage {
  role: 'user' | 'assistant'
  content: string
}

/** 会话摘要（列表项，不含 messages） */
export interface ChatSessionSummary {
  id: string
  title: string
  updatedAt: string
}

/** 完整会话（含 messages） */
export interface ChatSession {
  id: string
  title: string
  createdAt: string
  updatedAt: string
  messages: ChatMessage[]
}

/**
 * 前端可见的终端会话抽象。
 * - `onData`：Rust 通过 Tauri Channel 推送的 PTY 输出（已 flush 的批次）
 * - `write`：用户按键回写 PTY
 * - `close`：关闭并回收会话
 */
export interface TerminalSession {
  readonly id: number
  /** 关闭会话（幂等） */
  close(): Promise<void>
  /** 写入 PTY（按键或命令字符串） */
  write(data: string | Uint8Array): Promise<void>
  /** 通知后端窗口尺寸变化 */
  resize(cols: number, rows: number): Promise<void>
  /** 监听后端推送的输出 bytes */
  onOutput(handler: (bytes: Uint8Array) => void): () => void
  /** 监听会话结束（PTY EOF） */
  onExit(handler: () => void): () => void
}

export interface Bridge {
  readonly kind: 'browser' | 'tauri'
  listCapabilities(): Promise<Capability[]>
  viewDoc(cap: Capability): Promise<ViewResult>
  runScript(cap: Capability, args: string[]): Promise<RunResult>
  invokeSkill(cap: Capability, prompt: string): Promise<InvokeResult>
  checkUpdate(): Promise<UpdateInfo>
  openExternal(url: string): Promise<void>
  /** 弹文件选择框选 .elw.json 导入（写用户叠加层）。force 用于冲突覆盖。 */
  importCapability(force?: boolean): Promise<ImportResult>
  /** 弹保存框导出能力为 .elw.json。 */
  exportCapability(cap: Capability): Promise<ImportResult>
  /** 删除自定义能力（内置不可删）。 */
  deleteCapability(cap: Capability): Promise<ImportResult>
  /** 读取 LLM 配置生效视图（api_key 打码）。 */
  getLlmConfig(): Promise<LlmConfigInfo>
  /** 保存到用户层 ~/.elwright/config.json。apiKey=null 保留现值，空串=清除。 */
  setLlmConfig(baseUrl: string, apiKey: string | null, model: string): Promise<LlmConfigInfo>
  /** 用表单当前值测试连接（未保存也可测）。 */
  testLlmConnection(baseUrl: string, apiKey: string, model: string): Promise<string>
  /**
   * 多轮 AI 对话（非流式）：发送 user/assistant 历史，返回 assistant 回复。
   * system 提示词由后端固定前置；未配置/请求失败抛中文错误（对话无降级 SOP）。
   */
  chat(messages: ChatMessage[]): Promise<string>
  /** 列出本地会话摘要（按更新时间倒序）。 */
  listChatSessions(): Promise<ChatSessionSummary[]>
  /** 加载单个会话（含 messages）；不存在返回 null。 */
  loadChatSession(id: string): Promise<ChatSession | null>
  /** 保存（upsert）会话：id/title/messages。 */
  saveChatSession(id: string, title: string, messages: ChatMessage[]): Promise<void>
  /** 删除会话（幂等）。 */
  deleteChatSession(id: string): Promise<void>
  /**
   * 打开一个新终端会话。返回一个 TerminalSession：
   * - `onOutput` 接收 PTY 字节（原始 bytes，TUI 程序可能部分序列）
   * - `write` 写入按键/命令
   * - `close` 关闭会话
   * 仅桌面模式可用；浏览器预览模式直接抛错（与现有 import/export 保持一致）。
   */
  openTerminal(options?: { cwd?: string; shell?: string }): Promise<TerminalSession>
  /** 用户主目录（新终端默认 cwd）。浏览器预览返回 null。 */
  homeDir(): Promise<string | null>
}

const browserBridge: Bridge = {
  kind: 'browser',

  async listCapabilities() {
    const res = await fetch('/api/capabilities')
    if (!res.ok) throw new Error(`读取能力注册表失败: HTTP ${res.status}`)
    const data = await res.json()
    return (data.capabilities ?? []) as Capability[]
  },

  async viewDoc(cap) {
    const rel = cap.doc ?? cap.entry
    if (!rel) return { ok: false, content: '该能力没有可查看的文档' }
    const res = await fetch(`/api/file?path=${encodeURIComponent(rel)}`)
    if (!res.ok) return { ok: false, content: `文档不存在或不可读: ${rel}` }
    return { ok: true, content: await res.text(), path: rel }
  },

  async runScript(cap, args) {
    // 浏览器无法 spawn 子进程；真实执行走桌面壳 IPC 或 CLI：ew run <id>
    const cmd = `ew run ${cap.id}${args.length ? ' ' + args.join(' ') : ''}`
    return { ok: false, output: `【预览模式】浏览器无法执行脚本。\n真实运行请用 CLI：${cmd}` }
  },

  async invokeSkill(cap) {
    // 预览模式固定走降级路径（验证降级 UI）；真实 LLM 调用走桌面壳 IPC
    let content = '该技能型暂无离线 SOP，请联网并配置 LLM 后使用。'
    if (cap.degradeDoc) {
      const res = await fetch(`/api/file?path=${encodeURIComponent(cap.degradeDoc)}`)
      content = res.ok ? await res.text() : `SOP 文件不存在: ${cap.degradeDoc}`
    }
    return {
      source: 'degraded',
      content,
      note: '【预览模式】未接 LLM，固定展示离线降级 SOP',
    }
  },

  // 更新检查直接查 GitHub 公开 API（支持 CORS），预览与桌面行为一致
  async checkUpdate() {
    const res = await fetch('https://api.github.com/repos/X-85/Elwright/releases/latest', {
      headers: { Accept: 'application/vnd.github+json' },
    })
    if (!res.ok) throw new Error(`检查更新失败：GitHub 返回 HTTP ${res.status}`)
    const data = (await res.json()) as { tag_name: string; html_url: string }
    const current = __APP_VERSION__
    const latest = data.tag_name.replace(/^v/i, '')
    return {
      current,
      latest,
      updateAvailable: compareVersions(latest, current) > 0,
      releaseUrl: data.html_url,
    }
  },

  async openExternal(url) {
    window.open(url, '_blank', 'noopener')
  },

  async importCapability() {
    // 浏览器无法写用户叠加层目录；真实导入走桌面壳 dialog + IPC 或 CLI：ew import <file>
    return {
      ok: false,
      message: '【预览模式】浏览器无法写入用户叠加层。\n真实导入请用桌面应用或 CLI：ew import <文件.elw.json>',
    }
  },

  async exportCapability(cap) {
    // 浏览器无保存对话框，按 elwright-skill/0.1 格式组 bundle 走 Blob 下载。
    // 预览注册表全部视为 builtin，文件经 /api/file 读取（仅 resources/ 可见）。
    const files: { path: string; content: string }[] = []
    for (const rel of [cap.entry, cap.doc, cap.degradeDoc]) {
      if (!rel) continue
      const res = await fetch(`/api/file?path=${encodeURIComponent(rel)}`)
      if (res.ok) files.push({ path: rel, content: await res.text() })
    }
    const bundle = { schema: 'elwright-skill/0.1', capability: cap, files }
    const blob = new Blob([JSON.stringify(bundle, null, 2)], {
      type: 'application/json',
    })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `${cap.id}.elw.json`
    a.click()
    URL.revokeObjectURL(url)
    return { ok: true, message: `已导出 ${cap.id}（含 ${files.length} 个文件）` }
  },

  async deleteCapability() {
    return {
      ok: false,
      message: '【预览模式】浏览器无法删除用户叠加层条目。\n真实删除请用桌面应用或 CLI：ew delete <id>',
    }
  },

  // 浏览器无法读用户主目录的配置文件；真实读写走桌面壳 IPC 或 CLI：ew config
  async getLlmConfig() {
    throw new Error('【预览模式】浏览器无法读取用户配置。\n真实配置请用桌面应用的「模型设置」或 CLI：ew config')
  },

  async setLlmConfig() {
    throw new Error('【预览模式】浏览器无法写入用户配置。\n真实配置请用桌面应用的「模型设置」或 CLI：ew config set')
  },  async testLlmConnection(_baseUrl: string, _apiKey: string, _model: string) {
    // 连接测试本质是普通 HTTP POST，浏览器可直接发（CORS 由端点决定）
    const res = await fetch(`${_baseUrl.replace(/\/$/, '')}/chat/completions`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...(_apiKey ? { Authorization: `Bearer ${_apiKey}` } : {}),
      },
      body: JSON.stringify({
        model: _model,
        messages: [{ role: 'user', content: 'ping' }],
        max_tokens: 1,
      }),
    })
    if (!res.ok) throw new Error(`端点返回 HTTP ${res.status}：${(await res.text()).slice(0, 300)}`)
    return '连接正常（浏览器直连测试）'
  },

  async chat() {
    // 对话需要读用户配置链并经桌面壳前置 system 提示词；预览模式明确降级，不模拟
    throw new Error('【预览模式】浏览器无法发起 AI 对话。\n真实对话请用桌面应用。')
  },

  async listChatSessions() {
    // 浏览器无文件系统访问用户层；预览模式固定空列表（UI 显示「无会话」）
    return []
  },

  async loadChatSession() {
    return null
  },

  async saveChatSession() {
    // 静默丢弃——预览模式下会话不持久化，刷新即失（与 chat() 的降级口径一致）
  },

  async deleteChatSession() {
    // 同上，静默成功
  },

  async openTerminal() {
    throw new Error(
      '【预览模式】浏览器无法启动 PTY。\n真实终端请用桌面应用。',
    )
  },

  async homeDir() {
    // 浏览器预览无终端面板，不会走到这里；返回 null 保持接口完备
    return null
  },
}

async function tauriInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core')
  return invoke<T>(command, args)
}

const tauriBridge: Bridge = {
  kind: 'tauri',

  listCapabilities() {
    return tauriInvoke<Capability[]>('list_capabilities')
  },

  async viewDoc(cap) {
    try {
      return await tauriInvoke<ViewResult>('view_doc', { id: cap.id })
    } catch (error) {
      return { ok: false, content: `读取文档失败: ${messageOf(error)}` }
    }
  },

  async runScript(cap, args) {
    try {
      return await tauriInvoke<RunResult>('run_script', { id: cap.id, args })
    } catch (error) {
      return { ok: false, output: `执行失败: ${messageOf(error)}` }
    }
  },

  async invokeSkill(cap, prompt) {
    try {
      return await tauriInvoke<InvokeResult>('invoke_skill', { id: cap.id, prompt })
    } catch (error) {
      return {
        source: 'degraded',
        content: `调用失败: ${messageOf(error)}`,
        note: '桌面壳无法完成技能调用。',
      }
    }
  },

  checkUpdate() {
    return tauriInvoke<UpdateInfo>('check_update')
  },

  async openExternal(url) {
    const { openUrl } = await import('@tauri-apps/plugin-opener')
    await openUrl(url)
  },

  async importCapability(force = false) {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const path = await open({
      title: '选择要导入的能力文件',
      filters: [{ name: 'Elwright 能力包', extensions: ['json'] }],
      multiple: false,
    })
    if (!path) return { ok: false, message: '已取消导入' }
    try {
      const message = await tauriInvoke<string>('import_capability', {
        path,
        force,
      })
      return { ok: true, message }
    } catch (error) {
      const message = messageOf(error)
      return { ok: false, message, conflict: message.includes('已存在') }
    }
  },

  async exportCapability(cap) {
    const { save } = await import('@tauri-apps/plugin-dialog')
    const path = await save({
      title: '导出能力',
      defaultPath: `${cap.id}.elw.json`,
      filters: [{ name: 'Elwright 能力包', extensions: ['json'] }],
    })
    if (!path) return { ok: false, message: '已取消导出' }
    try {
      const message = await tauriInvoke<string>('export_capability', {
        id: cap.id,
        path,
      })
      return { ok: true, message }
    } catch (error) {
      return { ok: false, message: messageOf(error) }
    }
  },

  async deleteCapability(cap) {
    try {
      const message = await tauriInvoke<string>('delete_capability', {
        id: cap.id,
      })
      return { ok: true, message }
    } catch (error) {
      return { ok: false, message: messageOf(error) }
    }
  },

  async getLlmConfig() {
    const raw = await tauriInvoke<Record<string, unknown>>('get_llm_config')
    // Rust snake_case → 前端 camelCase
    return {
      baseUrl: String(raw.base_url ?? ''),
      model: String(raw.model ?? ''),
      apiKeyMasked: String(raw.api_key_masked ?? ''),
      source: [
        String(raw.source?.[0] ?? ''),
        String(raw.source?.[1] ?? ''),
        String(raw.source?.[2] ?? ''),
      ],
      userConfigPath: raw.user_config_path ? String(raw.user_config_path) : undefined,
    }
  },

  async setLlmConfig(baseUrl: string, apiKey: string | null, model: string) {
    const raw = await tauriInvoke<Record<string, unknown>>('set_llm_config', {
      baseUrl,
      apiKey, // null 序列化为 JSON null → Rust Option::None（保留现值）
      model,
    })
    return {
      baseUrl: String(raw.base_url ?? ''),
      model: String(raw.model ?? ''),
      apiKeyMasked: String(raw.api_key_masked ?? ''),
      source: [
        String(raw.source?.[0] ?? ''),
        String(raw.source?.[1] ?? ''),
        String(raw.source?.[2] ?? ''),
      ],
      userConfigPath: raw.user_config_path ? String(raw.user_config_path) : undefined,
    }
  },

  async testLlmConnection(baseUrl, apiKey, model) {
    return tauriInvoke<string>('test_llm_connection', { baseUrl, apiKey, model })
  },

  async chat(messages) {
    return tauriInvoke<string>('chat_completion', { messages })
  },

  async listChatSessions() {
    const raw = await tauriInvoke<unknown[]>('chat_list_sessions')
    return (raw as Record<string, unknown>[]).map((r) => ({
      id: String(r.id ?? ''),
      title: String(r.title ?? ''),
      updatedAt: String(r.updated_at ?? ''),
    }))
  },

  async loadChatSession(id) {
    const raw = (await tauriInvoke<Record<string, unknown> | null>('chat_load_session', { id })) as
      | Record<string, unknown>
      | null
    if (!raw) return null
    const msgs = (raw.messages as Record<string, unknown>[] | undefined) ?? []
    return {
      id: String(raw.id ?? ''),
      title: String(raw.title ?? ''),
      createdAt: String(raw.created_at ?? ''),
      updatedAt: String(raw.updated_at ?? ''),
      messages: msgs.map((m) => ({
        role: (String(m.role ?? 'user') === 'assistant' ? 'assistant' : 'user') as 'user' | 'assistant',
        content: String(m.content ?? ''),
      })),
    }
  },

  async saveChatSession(id, title, messages) {
    await tauriInvoke<void>('chat_save_session', { id, title, messages })
  },

  async deleteChatSession(id) {
    await tauriInvoke<void>('chat_delete_session', { id })
  },

  async openTerminal(options) {
    const { Channel } = await import('@tauri-apps/api/core')
    const channel = new Channel<number[]>()
    let onOutput: ((bytes: Uint8Array) => void) | null = null
    let onExit: (() => void) | null = null
    let closed = false
    channel.onmessage = (raw) => {
      const bytes = raw instanceof Uint8Array ? raw : new Uint8Array(raw)
      // 后端在 PTY 关闭时不再 send bytes，但仍要触发 exit
      onOutput?.(bytes)
    }
    // cols/rows 由前端 xterm.js 提供初值，IPC 返回 id 后前端会立即 resize。
    // channel 作为参数传入：Tauri 把它解析回指向 onmessage 的回调，
    // 后端 PTY 输出经此推给前端（返回值只带 id）。
    const cols = 80
    const rows = 24
    const id = await tauriInvoke<number>('terminal_open', {
      cols,
      rows,
      cwd: options?.cwd ?? null,
      shell: options?.shell ?? null,
      env: null,
      channel,
    })
    const session: TerminalSession = {
      id,
      async close() {
        if (closed) return
        closed = true
        try {
          await tauriInvoke('terminal_close', { id })
        } catch {
          // best-effort
        }
        onExit?.()
      },
      async write(data) {
        if (closed) throw new Error('终端会话已关闭')
        const bytes = typeof data === 'string' ? new TextEncoder().encode(data) : data
        await tauriInvoke('terminal_write', { id, data: Array.from(bytes) })
      },
      async resize(c, r) {
        if (closed) return
        try {
          await tauriInvoke('terminal_resize', { id, cols: c, rows: r })
        } catch {
          // ignore: PTY 已退出
        }
      },
      onOutput(handler) {
        onOutput = handler
        return () => {
          if (onOutput === handler) onOutput = null
        }
      },
      onExit(handler) {
        onExit = handler
        return () => {
          if (onExit === handler) onExit = null
        }
      },
    }
    return session
  },

  async homeDir() {
    const { homeDir } = await import('@tauri-apps/api/path')
    return homeDir()
  },
}

/// 语义与 core::version::is_newer 一致（core 有单测覆盖）：逐段数值比较，
/// 段内非数字后缀取前导数字，缺段视为 0。返回 >0 表示 a 更新。
/// 导出供单测（纯函数，无副作用）。
export function compareVersions(a: string, b: string): number {
  const seg = (v: string) =>
    v
      .replace(/^v/i, '')
      .split(/[.-]/)
      .map((p) => parseInt(p, 10) || 0)
  const x = seg(a)
  const y = seg(b)
  for (let i = 0; i < Math.max(x.length, y.length); i++) {
    const d = (x[i] ?? 0) - (y[i] ?? 0)
    if (d !== 0) return d
  }
  return 0
}

function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

// vite define 注入（vite.config.ts），与 tauri.conf.json 的 version 保持同步
declare const __APP_VERSION__: string

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown
  }
}

export function createBridge(): Bridge {
  return window.__TAURI_INTERNALS__ ? tauriBridge : browserBridge
}
