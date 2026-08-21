/// 桌面壳与核心的边界抽象：UI 只依赖 Bridge 接口，不关心底下是
/// 浏览器预览（dev 中间件 /api/*）还是 Tauri IPC。
/// 阶段 3b 接入 Tauri 时新增 tauriBridge 适配器即可，UI 零改动。

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

// TODO(阶段3b): tauriBridge —— window.__TAURI_INTERNALS__ 存在时用
// @tauri-apps/api 的 invoke() 调 src-tauri 侧命令：
//   list_capabilities / view_doc / run_script / invoke_skill
declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown
  }
}

export function createBridge(): Bridge {
  return browserBridge
}
