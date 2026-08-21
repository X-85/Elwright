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

export interface Bridge {
  readonly kind: 'browser' | 'tauri'
  listCapabilities(): Promise<Capability[]>
  viewDoc(cap: Capability): Promise<ViewResult>
  runScript(cap: Capability, args: string[]): Promise<RunResult>
  invokeSkill(cap: Capability, prompt: string): Promise<InvokeResult>
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
}

function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown
  }
}

export function createBridge(): Bridge {
  return window.__TAURI_INTERNALS__ ? tauriBridge : browserBridge
}
