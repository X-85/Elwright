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
  /** 渐进式发布档位，1 为核心能力。 */
  releaseTier?: number
  /** 本地使用达到该次数后解锁；未配置表示按档位可见。 */
  unlockAfterUses?: number
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

/** 命名模型档案元信息（设置中心 模型设置下拉用） */
export interface LlmProfileMeta {
  name: string
  active: boolean
  source: 'user' | string
}

/** 前端写入/编辑档案用的完整结构（不含 active 状态） */
export interface LlmProfileInput {
  name: string
  baseUrl: string
  apiKey: string
  model: string
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

/** 工作工具栏 · Todo 条目（与 core::workbench::TodoItem 对应，camelCase） */
export interface TodoItem {
  id: number
  text: string
  done: boolean
  createdAt: string
  completedAt: string | null
}

/** 工作工具栏 · 今日记录日期（YYYY-MM-DD）列表 */
export type NoteDate = string
export interface WorkspaceFolder {
  id: string
  name: string
  parentId: string | null
}

export interface WorkspaceResource {
  id: string
  title: string
  kind: 'url' | 'path' | 'capability' | 'note' | 'app' | string
  value: string
  folderId: string | null
  note: string
  launchArgs: string[]
  icon: string
}

export interface WorkspaceTopic {
  id: string
  title: string
  question: string
  resourceIds: string[]
  report: string
  updatedAt: string
}

export interface WorkspaceData {
  folders: WorkspaceFolder[]
  resources: WorkspaceResource[]
  topics: WorkspaceTopic[]
}

export interface TopicReportResult {
  source: 'llm' | 'offline' | string
  content: string
  note?: string
}


// ---- 代码浏览器阶段①（feature-2026-08-code-browser-phase1）----

/** 目录树条目。 */
export interface CodeTreeEntry {
  path: string
  name: string
  kind: 'dir' | 'file'
  size: number
  readable: boolean
  sensitive: boolean
}

/** 打开的代码文档。 */
export interface CodeDocument {
  path: string
  name: string
  language: string
  size: number
  truncated: boolean
  content: string
  sensitive: boolean
  notice: string
}

/** 搜索命中。 */
export interface CodeSearchHit {
  path: string
  name: string
  line: number
  snippet: string
}

/** 轻量符号命中。 */
export interface CodeSymbolHit {
  name: string
  kind: 'class' | 'interface' | 'enum' | 'record' | 'method' | string
  path: string
  line: number
  declaration: string
}

/** 最近项目 / 最近文件 / 收藏 / 书签持久化结构。 */
export interface CodeBrowserRecent {
  projects: { name: string; rootPath: string; lastOpenedAt: number }[]
  files: { projectRoot: string; path: string; lastOpenedAt: number }[]
  favorites: { projectRoot: string; path: string }[]
  bookmarks: { projectRoot: string; path: string; line: number; label: string }[]
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
  /** 列出全部命名档案（Q19 模型设置下拉用）。 */
  listLlmProfiles(): Promise<LlmProfileMeta[]>
  /** 当前激活的档案名；null = 走 flat 字段。 */
  getActiveLlmProfile(): Promise<string | null>
  /** 切换激活档案（不存在时拒绝）。 */
  setActiveLlmProfile(name: string): Promise<void>
  /** 新建/覆盖档案（name 校验由后端负责）。 */
  saveLlmProfile(profile: LlmProfileInput): Promise<void>
  /** 删除档案（若当前激活则自动回退 flat）。 */
  deleteLlmProfile(name: string): Promise<void>
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
  // ---- 工作工具栏（Todo + 今日记录）----
  /** 全量 Todo（创建序）。 */
  todoList(): Promise<TodoItem[]>
  /** 新增一条，返回完整条目。 */
  todoAdd(text: string): Promise<TodoItem>
  /** 勾选/取消勾选。 */
  todoToggle(id: number): Promise<TodoItem>
  /** 删除。 */
  todoRemove(id: number): Promise<void>
  /** 读某日记录；无记录返回 null。 */
  noteGet(date: string): Promise<string | null>
  /** 保存某日记录（整文件覆盖）。 */
  noteSave(date: string, content: string): Promise<void>
  /** 已有记录的日期列表（倒序）。 */
  noteList(): Promise<string[]>,
  loadWorkspace(): Promise<WorkspaceData>
  createWorkspaceFolder(name: string, parentId: string | null): Promise<WorkspaceFolder>
  deleteWorkspaceFolder(id: string): Promise<void>
  /** 选择一个本地文件，桌面模式返回绝对路径；浏览器预览返回 null。 */
  chooseWorkspaceFile(): Promise<string | null>
  createWorkspaceResource(resource: Omit<WorkspaceResource, 'id'>): Promise<WorkspaceResource>
  deleteWorkspaceResource(id: string): Promise<void>
  launchWorkspaceApp(id: string): Promise<string>
  createWorkspaceTopic(title: string, question: string): Promise<WorkspaceTopic>
  updateWorkspaceTopic(topic: WorkspaceTopic): Promise<void>
  deleteWorkspaceTopic(id: string): Promise<void>
  generateTopicReport(id: string): Promise<TopicReportResult>
  // ---- 代码浏览器（只读；浏览器预览端明确降级，不伪造文件访问）----
  /** 打开系统目录选择器选项目根；浏览器预览返回 null 并提示。 */
  chooseProjectDirectory(): Promise<string | null>
  codeBrowserTree(projectRoot: string, rel: string): Promise<CodeTreeEntry[]>
  codeBrowserRead(projectRoot: string, rel: string): Promise<CodeDocument>
  codeBrowserSearch(projectRoot: string, query: string, mode: 'filename' | 'content'): Promise<CodeSearchHit[]>
  codeBrowserScanSymbols(projectRoot: string): Promise<CodeSymbolHit[]>
  codeBrowserRecentLoad(): Promise<CodeBrowserRecent>
  /** 记录一次打开（项目 / 项目+文件），返回更新后的最近列表。 */
  codeBrowserRecentOpen(projectRoot: string, rel: string): Promise<CodeBrowserRecent>
  /** 删除一条最近项目（含其名下最近文件；收藏/书签保留），返回更新后的最近列表。 */
  codeBrowserRecentRemoveProject(projectRoot: string): Promise<CodeBrowserRecent>
  /** 流式对话（阶段④，仅桌面）：事件 type = delta | done | error | cancelled。 */
  chatCompletionStream(
    requestId: number,
    messages: ChatMessage[],
    onEvent: (e: { type: 'delta' | 'done' | 'error' | 'cancelled'; text?: string; message?: string }) => void,
  ): Promise<void>
  /** 取消在途流式请求（后端中断读取）。 */
  chatCancel(requestId: number): Promise<void>
  /** 切换收藏文件，返回更新后的收藏列表。 */
  codeBrowserFavoritesToggle(projectRoot: string, rel: string): Promise<CodeBrowserRecent['favorites']>
  /** 切换代码书签（同路径同行去重），返回更新后的书签列表。 */
  codeBrowserBookmarksToggle(projectRoot: string, rel: string, line: number, label: string): Promise<CodeBrowserRecent['bookmarks']>
  /**
   * 解析 unified diff 并预览（不写文件）。warnings 含敏感路径拒收等提示。
   */
  applyPatchPreview(projectRoot: string, patchText: string): Promise<unknown>
  /**
   * 应用预览到项目内文件。previews 由前端三栏对话框逐 hunk 选择后回传。
   * 返回 { applied, skipped, snapshot_id }。
   */
  applyPatchApply(projectRoot: string, previews: unknown[]): Promise<unknown>
  /**
   * 撤销一次已应用补丁：按快照 ID 写回原内容。
   */
  applyPatchRevert(projectRoot: string, snapshotId: string): Promise<unknown>
  /**
   * 列出当前项目所有未撤销快照（UI 撤销列表用）。
   */
  applyPatchSnapshots(projectRoot: string): Promise<unknown[]>
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
  },

  // Q19 模型档案（预览模式全部抛明确降级）
  async listLlmProfiles() {
    throw new Error('【预览模式】浏览器无法读取模型档案。\n真实操作请用桌面应用的「模型设置」或 CLI：ew config profile list')
  },
  async getActiveLlmProfile() {
    throw new Error('【预览模式】浏览器无法读取激活档案。\n真实操作请用桌面应用的「模型设置」')
  },
  async setActiveLlmProfile() {
    throw new Error('【预览模式】浏览器无法切换档案。\n真实操作请用桌面应用的「模型设置」或 CLI：ew config profile use <name>')
  },
  async saveLlmProfile() {
    throw new Error('【预览模式】浏览器无法保存档案。\n真实操作请用桌面应用的「模型设置」或 CLI：ew config profile add')
  },
  async deleteLlmProfile() {
    throw new Error('【预览模式】浏览器无法删除档案。\n真实操作请用桌面应用的「模型设置」或 CLI：ew config profile remove')
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

  // 工作工具栏：进程内模拟存储——UI 可在浏览器预览完整体验，
  // 但不持久化（刷新即失），真实数据走桌面壳 IPC 写 ~/.elwright/。
  async todoList() {
    return [...browserWorkbenchTodos]
  },

  async todoAdd(text) {
    const item: TodoItem = {
      id: browserWorkbenchNextId++,
      text,
      done: false,
      createdAt: new Date().toISOString(),
      completedAt: null,
    }
    browserWorkbenchTodos.push(item)
    return { ...item }
  },

  async todoToggle(id) {
    const item = browserWorkbenchTodos.find((t) => t.id === id)
    if (!item) throw new Error(`Todo ${id} 不存在或已删除`)
    item.done = !item.done
    item.completedAt = item.done ? new Date().toISOString() : null
    return { ...item }
  },

  async todoRemove(id) {
    const idx = browserWorkbenchTodos.findIndex((t) => t.id === id)
    if (idx === -1) throw new Error(`Todo ${id} 不存在或已删除`)
    browserWorkbenchTodos.splice(idx, 1)
  },

  async noteGet(date) {
    return browserWorkbenchNotes.get(date) ?? null
  },

  async noteSave(date, content) {
    if (!/^\d{4}-\d{2}-\d{2}$/.test(date)) throw new Error(`日期格式无效（应为 YYYY-MM-DD）: ${date}`)
    browserWorkbenchNotes.set(date, content)
  },

  async noteList() {
    return [...browserWorkbenchNotes.keys()].sort().reverse()
  },

  async loadWorkspace() {
    const raw = localStorage.getItem('elwright-workspace')
    if (!raw) return { folders: [], resources: [], topics: [] }
    try {
      const data = JSON.parse(raw) as WorkspaceData
      data.folders ??= []
      data.resources = (data.resources ?? []).map((resource) => ({
        ...resource,
        folderId: resource.folderId ?? null,
        note: resource.note ?? '',
        launchArgs: resource.launchArgs ?? [],
        icon: resource.icon ?? '',
      }))
      data.topics = (data.topics ?? []).map((topic) => ({
        ...topic,
        resourceIds: topic.resourceIds ?? [],
        report: topic.report ?? '',
      }))
      return data
    } catch {
      return { folders: [], resources: [], topics: [] }
    }
  },

  async createWorkspaceFolder(name, parentId) {
    const data = await this.loadWorkspace()
    let depth = 1
    let parent = parentId
    while (parent) {
      const folder = data.folders.find((f) => f.id === parent)
      if (!folder) throw new Error('父文件夹不存在')
      depth += 1
      parent = folder.parentId
    }
    if (depth > 3) throw new Error('文件夹最多支持三层嵌套')
    const folder = { id: `folder-${Date.now()}-${Math.random().toString(16).slice(2)}`, name: name.trim(), parentId }
    if (!folder.name) throw new Error('文件夹名称不能为空')
    data.folders.push(folder)
    localStorage.setItem('elwright-workspace', JSON.stringify(data))
    return folder
  },

  async deleteWorkspaceFolder(id) {
    const data = await this.loadWorkspace()
    const removed = new Set([id])
    let changed = true
    while (changed) {
      changed = false
      data.folders.forEach((f) => { if (f.parentId && removed.has(f.parentId) && !removed.has(f.id)) { removed.add(f.id); changed = true } })
    }
    data.folders = data.folders.filter((f) => !removed.has(f.id))
    data.resources.forEach((r) => { if (r.folderId && removed.has(r.folderId)) r.folderId = null })
    localStorage.setItem('elwright-workspace', JSON.stringify(data))
  },

  async chooseWorkspaceFile() {
    return null
  },

  async chooseProjectDirectory() {
    return null
  },

  async codeBrowserTree() {
    throw new Error('【预览模式】浏览器无法读取本机项目文件，请在桌面应用中使用代码浏览器。')
  },

  async codeBrowserRead() {
    throw new Error('【预览模式】浏览器无法读取本机项目文件，请在桌面应用中使用代码浏览器。')
  },

  async codeBrowserSearch() {
    throw new Error('【预览模式】浏览器无法搜索本机项目文件，请在桌面应用中使用代码浏览器。')
  },

  async codeBrowserScanSymbols() {
    throw new Error('【预览模式】浏览器无法扫描本机项目文件，请在桌面应用中使用代码浏览器。')
  },

  async codeBrowserRecentLoad() {
    return { projects: [], files: [] }
  },

  async codeBrowserRecentOpen() {
    // 浏览器端不落盘，返回空记录，不伪造持久化。
    return { projects: [], files: [], favorites: [], bookmarks: [] }
  },

  async codeBrowserRecentRemoveProject() {
    // 浏览器端最近列表本为空，删除是无害空操作。
    return { projects: [], files: [], favorites: [], bookmarks: [] }
  },

  async chatCompletionStream() {
    throw new Error('【预览模式】AI 对话仅在桌面应用可用。')
  },

  async chatCancel() {
    // 浏览器无在途桌面请求，静默忽略
  },

  async codeBrowserFavoritesToggle() {
    throw new Error('【预览模式】收藏需桌面端持久化，请在桌面应用中使用。')
  },

  async codeBrowserBookmarksToggle() {
    throw new Error('【预览模式】书签需桌面端持久化，请在桌面应用中使用。')
  },

  async applyPatchPreview() {
    throw new Error('【预览模式】补丁预览需桌面端，请在桌面应用中使用。')
  },

  async applyPatchApply() {
    throw new Error('【预览模式】补丁应用需桌面端，请在桌面应用中使用。')
  },

  async applyPatchRevert() {
    throw new Error('【预览模式】补丁撤销需桌面端，请在桌面应用中使用。')
  },

  async applyPatchSnapshots() {
    return []
  },

  async createWorkspaceResource(resource) {
    const data = await this.loadWorkspace()
    if (!resource.title.trim() || !resource.value.trim()) throw new Error('资源标题和内容不能为空')
    if (resource.folderId && !data.folders.some((f) => f.id === resource.folderId)) throw new Error('目标文件夹不存在')
    const created = { ...resource, id: `resource-${Date.now()}-${Math.random().toString(16).slice(2)}` }
    data.resources.push(created)
    localStorage.setItem('elwright-workspace', JSON.stringify(data))
    return created
  },

  async deleteWorkspaceResource(id) {
    const data = await this.loadWorkspace()
    data.resources = data.resources.filter((r) => r.id !== id)
    data.topics.forEach((t) => { t.resourceIds = t.resourceIds.filter((rid) => rid !== id) })
    localStorage.setItem('elwright-workspace', JSON.stringify(data))
  },

  async launchWorkspaceApp() {
    throw new Error('【预览模式】浏览器无法启动本机软件，请在桌面应用中打开快捷方式。')
  },

  async createWorkspaceTopic(title, question) {
    const data = await this.loadWorkspace()
    if (!title.trim()) throw new Error('课题名称不能为空')
    const topic = { id: `topic-${Date.now()}-${Math.random().toString(16).slice(2)}`, title: title.trim(), question: question.trim(), resourceIds: [], report: '', updatedAt: new Date().toISOString() }
    data.topics.push(topic)
    localStorage.setItem('elwright-workspace', JSON.stringify(data))
    return topic
  },

  async updateWorkspaceTopic(topic) {
    const data = await this.loadWorkspace()
    const index = data.topics.findIndex((t) => t.id === topic.id)
    if (index < 0) throw new Error('课题不存在')
    data.topics[index] = { ...topic, updatedAt: new Date().toISOString() }
    localStorage.setItem('elwright-workspace', JSON.stringify(data))
  },

  async deleteWorkspaceTopic(id) {
    const data = await this.loadWorkspace()
    data.topics = data.topics.filter((t) => t.id !== id)
    localStorage.setItem('elwright-workspace', JSON.stringify(data))
  },

  async generateTopicReport(id) {
    const data = await this.loadWorkspace()
    const topic = data.topics.find((t) => t.id === id)
    if (!topic) throw new Error('课题不存在')
    const resources = topic.resourceIds.map((rid) => data.resources.find((r) => r.id === rid)).filter(Boolean) as WorkspaceResource[]
    const source = resources.map((r) => `### ${r.title} [${r.kind}]\n${r.value}${r.note ? `\n备注：${r.note}` : ''}`).join('\n\n')
    const content = `# ${topic.title}\n\n## 研究问题\n${topic.question || '（未填写）'}\n\n## 当前资料\n${source || '暂无已关联资源。'}\n\n## 分析框架\n1. 明确核心概念与边界。\n2. 对照资料中的事实、示例和限制。\n3. 将结论拆解为可验证的实践步骤。\n\n## 待补充\n- 为每个关键判断补充原始出处与反例。\n- 用实际案例验证结论，并记录版本与环境。\n\n> 这是预览模式下的离线报告草稿。`
    topic.report = content
    await this.updateWorkspaceTopic(topic)
    return { source: 'offline', content, note: '【预览模式】未接 LLM，已生成离线报告草稿。' }
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

// browserBridge 工作工具栏的进程内模拟存储（模块级，刷新即重置）
const browserWorkbenchTodos: TodoItem[] = []
let browserWorkbenchNextId = 1
const browserWorkbenchNotes = new Map<string, string>()

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

  // Q19 模型档案
  async listLlmProfiles() {
    const raw = await tauriInvoke<Array<Record<string, unknown>>>('llm_list_profiles')
    return raw.map((r) => ({
      name: String(r.name ?? ''),
      active: Boolean(r.active),
      source: String(r.source ?? 'user'),
    }))
  },
  async getActiveLlmProfile() {
    const raw = await tauriInvoke<string | null>('llm_get_active_profile')
    return raw ?? null
  },
  async setActiveLlmProfile(name) {
    await tauriInvoke<void>('llm_set_active_profile', { name })
  },
  async saveLlmProfile(profile) {
    await tauriInvoke<void>('llm_save_profile', {
      profile: {
        name: profile.name,
        base_url: profile.baseUrl,
        api_key: profile.apiKey,
        model: profile.model,
      },
    })
  },
  async deleteLlmProfile(name) {
    await tauriInvoke<void>('llm_delete_profile', { name })
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

  // 工作工具栏：真实 IPC（core::workbench，camelCase 自动对齐）
  async todoList() {
    return tauriInvoke<TodoItem[]>('todo_list')
  },

  async todoAdd(text) {
    return tauriInvoke<TodoItem>('todo_add', { text })
  },

  async todoToggle(id) {
    return tauriInvoke<TodoItem>('todo_toggle', { id })
  },

  async todoRemove(id) {
    await tauriInvoke<void>('todo_remove', { id })
  },

  async noteGet(date) {
    return tauriInvoke<string | null>('note_get', { date })
  },

  async noteSave(date, content) {
    await tauriInvoke<void>('note_save', { date, content })
  },

  async noteList() {
    return tauriInvoke<string[]>('note_list')
  },

  loadWorkspace() {
    return tauriInvoke<WorkspaceData>('workspace_load')
  },

  createWorkspaceFolder(name, parentId) {
    return tauriInvoke<WorkspaceFolder>('workspace_create_folder', { name, parentId })
  },

  async deleteWorkspaceFolder(id) {
    await tauriInvoke<void>('workspace_delete_folder', { id })
  },

  async chooseWorkspaceFile() {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const path = await open({ title: '选择要收藏的本地文件', multiple: false, directory: false })
    return typeof path === 'string' ? path : null
  },

  async chooseProjectDirectory() {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const path = await open({ title: '选择要浏览的项目目录', multiple: false, directory: true })
    return typeof path === 'string' ? path : null
  },

  codeBrowserTree(projectRoot, rel) {
    return tauriInvoke<CodeTreeEntry[]>('code_browser_tree', { projectRoot, rel })
  },

  codeBrowserRead(projectRoot, rel) {
    return tauriInvoke<CodeDocument>('code_browser_read', { projectRoot, rel })
  },

  codeBrowserSearch(projectRoot, query, mode) {
    return tauriInvoke<CodeSearchHit[]>('code_browser_search', { projectRoot, query, mode })
  },

  codeBrowserScanSymbols(projectRoot) {
    return tauriInvoke<CodeSymbolHit[]>('code_browser_scan_symbols', { projectRoot })
  },

  codeBrowserRecentLoad() {
    return tauriInvoke<CodeBrowserRecent>('code_browser_recent_load', {})
  },

  codeBrowserRecentOpen(projectRoot, rel) {
    return tauriInvoke<CodeBrowserRecent>('code_browser_recent_open', { projectRoot, rel })
  },

  codeBrowserRecentRemoveProject(projectRoot) {
    return tauriInvoke<CodeBrowserRecent>('code_browser_recent_remove_project', { projectRoot })
  },

  async chatCompletionStream(requestId, messages, onEvent) {
    const { Channel, invoke } = await import('@tauri-apps/api/core')
    const channel = new Channel<string>((raw) => {
      try {
        onEvent(JSON.parse(typeof raw === 'string' ? raw : String(raw)))
      } catch {
        // 非 JSON 事件忽略
      }
    })
    await invoke('chat_completion_stream', { requestId, messages, channel })
  },

  async chatCancel(requestId) {
    await tauriInvoke<void>('chat_cancel', { requestId })
  },

  codeBrowserFavoritesToggle(projectRoot, rel) {
    return tauriInvoke<CodeBrowserRecent['favorites']>('code_browser_favorites_toggle', { projectRoot, rel })
  },

  codeBrowserBookmarksToggle(projectRoot, rel, line, label) {
    return tauriInvoke<CodeBrowserRecent['bookmarks']>('code_browser_bookmarks_toggle', { projectRoot, rel, line, label })
  },

  applyPatchPreview(projectRoot, patchText) {
    return tauriInvoke<unknown>('apply_patch_preview', { projectRoot, patchText })
  },

  applyPatchApply(projectRoot, previews) {
    return tauriInvoke<unknown>('apply_patch_apply', { projectRoot, previews })
  },

  applyPatchRevert(projectRoot, snapshotId) {
    return tauriInvoke<unknown>('apply_patch_revert', { projectRoot, snapshotId })
  },

  applyPatchSnapshots(projectRoot) {
    return tauriInvoke<unknown[]>('apply_patch_snapshots', { projectRoot })
  },

  createWorkspaceResource(resource) {
    return tauriInvoke<WorkspaceResource>('workspace_create_resource', { resource })
  },

  async deleteWorkspaceResource(id) {
    await tauriInvoke<void>('workspace_delete_resource', { id })
  },

  launchWorkspaceApp(id) {
    return tauriInvoke<string>('workspace_launch_app', { id })
  },

  createWorkspaceTopic(title, question) {
    return tauriInvoke<WorkspaceTopic>('workspace_create_topic', { title, question })
  },

  async updateWorkspaceTopic(topic) {
    await tauriInvoke<void>('workspace_update_topic', { topic })
  },

  async deleteWorkspaceTopic(id) {
    await tauriInvoke<void>('workspace_delete_topic', { id })
  },

  generateTopicReport(id) {
    return tauriInvoke<TopicReportResult>('workspace_generate_report', { id })
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
