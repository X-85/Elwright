/**
 * Todo 联动（代码浏览器阶段③第二批）：
 * Todo 文本里 `绝对路径:行号` 形式的代码位置标记的解析与渲染辅助。
 * 标记由代码浏览器「转为 Todo」写入（反引号包裹，避免误伤普通文本）。
 */

export interface CodeLinkPart {
  kind: 'text' | 'code'
  text: string
  path?: string
  line?: number
}

const MARK_RE = /`([^`\n]+):(\d+)`/g

/** 把 Todo 文本拆成普通片段与代码位置片段；解析失败的原样保留。 */
export function parseCodeLinks(text: string): CodeLinkPart[] {
  const parts: CodeLinkPart[] = []
  let last = 0
  for (const m of text.matchAll(MARK_RE)) {
    const idx = m.index ?? 0
    if (idx > last) parts.push({ kind: 'text', text: text.slice(last, idx) })
    parts.push({ kind: 'code', text: m[0], path: m[1], line: Number(m[2]) })
    last = idx + m[0].length
  }
  if (last < text.length) parts.push({ kind: 'text', text: text.slice(last) })
  return parts
}

/** 代码浏览器侧生成标记文本：`绝对路径:行号`（反引号包裹）。 */
export function codeLinkMarker(absPath: string, line: number): string {
  return '`' + absPath + ':' + line + '`'
}
