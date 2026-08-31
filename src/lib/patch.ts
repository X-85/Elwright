//! 补丁前端辅助：识别 AI 回复中的 ```diff``` 块、构建预览三栏文案。
//! 后端 patch::PatchPreview 是权威，本模块只用于前端识别 diff 文本与触发入口。

/** 从助手消息全文里抽取第一个 ```diff``` 块的文本（含 ---/+++ 头与 hunk）。 */
export function extractFirstDiff(content: string): string | null {
  // 匹配 ```diff 或 ``` 紧跟 diff 语言的围栏块
  const re = /```(?:diff)?\s*\n([\s\S]*?)```/g
  let m: RegExpExecArray | null
  while ((m = re.exec(content)) !== null) {
    const body = m[1]
    if (body.includes('--- ') && body.includes('+++ ') && body.includes('@@')) {
      return body
    }
  }
  return null
}

/** 简单 diff 行渲染（前端预览用）：红删绿增。 */
export interface DiffLine {
  kind: 'add' | 'del' | 'ctx'
  text: string
}

export function renderDiffLines(diff: string): DiffLine[] {
  const out: DiffLine[] = []
  for (const raw of diff.split('\n')) {
    if (raw.startsWith('@@') || raw.startsWith('---') || raw.startsWith('+++')) continue
    if (raw.startsWith('+')) out.push({ kind: 'add', text: raw.slice(1) })
    else if (raw.startsWith('-')) out.push({ kind: 'del', text: raw.slice(1) })
    else if (raw.startsWith(' ') || raw === '') out.push({ kind: 'ctx', text: raw.slice(1) })
  }
  return out
}

/** 三栏预览渲染后的产物（PatchPreviewDialog 用）。 */
export interface PreviewFile {
  file: string
  currentContent: string
  newContent: string
  hunks: unknown[]
  rejected: boolean
}

export interface PatchPreview {
  files: PreviewFile[]
  warnings: string[]
}