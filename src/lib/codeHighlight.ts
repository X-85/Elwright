/**
 * 代码浏览器阶段①的轻量高亮：先整体 HTML 转义，再做有限的关键词/
 * 字符串/注释着色，杜绝把未转义内容塞进 v-html（ADR-002 同源思路）。
 * 只做行内 token 级着色，不做完整语法树。
 */

export function escapeHtml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

const KEYWORDS: Record<string, string[]> = {
  java: ['public', 'private', 'protected', 'class', 'interface', 'enum', 'record', 'extends', 'implements', 'static', 'final', 'void', 'return', 'new', 'import', 'package', 'this', 'super', 'throws', 'try', 'catch', 'finally', 'if', 'else', 'for', 'while', 'switch', 'case', 'break', 'continue', 'abstract', 'synchronized', 'default', 'null', 'true', 'false'],
  kotlin: ['fun', 'val', 'var', 'class', 'object', 'interface', 'extends', 'implements', 'return', 'if', 'else', 'for', 'while', 'null', 'true', 'false'],
  javascript: ['function', 'const', 'let', 'var', 'return', 'if', 'else', 'for', 'while', 'class', 'import', 'export', 'from', 'new', 'this', 'null', 'true', 'false', 'async', 'await'],
  typescript: ['function', 'const', 'let', 'var', 'return', 'if', 'else', 'for', 'while', 'class', 'interface', 'type', 'import', 'export', 'from', 'new', 'this', 'null', 'true', 'false', 'async', 'await', 'readonly', 'enum'],
  python: ['def', 'class', 'return', 'if', 'elif', 'else', 'for', 'while', 'import', 'from', 'as', 'with', 'try', 'except', 'finally', 'None', 'True', 'False', 'lambda', 'pass', 'raise'],
  shell: ['if', 'then', 'fi', 'for', 'do', 'done', 'while', 'case', 'esac', 'function', 'echo', 'export', 'exit', 'return'],
  sql: ['select', 'from', 'where', 'insert', 'into', 'update', 'delete', 'join', 'left', 'right', 'inner', 'group', 'order', 'by', 'having', 'limit', 'create', 'table', 'alter', 'drop', 'index'],
  toml: [],
  yaml: [],
  properties: [],
  json: ['true', 'false', 'null'],
  markdown: [],
  xml: [],
  html: [],
  css: [],
  gradle: [],
  text: [],
  binary: [],
}

/** 行注释前缀（按语言）；块注释仅 java/java 系做跨行简化处理。 */
const LINE_COMMENTS: Record<string, string[]> = {
  java: ['//'], kotlin: ['//'], javascript: ['//'], typescript: ['//'],
  shell: ['#'], python: ['#'], yaml: ['#'], properties: ['#', '!'], toml: ['#'],
  gradle: ['//'], sql: ['--'], text: [],
  json: [], markdown: [], xml: [], html: [], css: [], binary: [],
}

/**
 * 高亮一段已知的纯文本代码，返回可直接用于 v-html 的 HTML。
 * 过程：整段先转义 → 再在转义后的文本上匹配 token（正则只命中
 * 转义后的安全字符集合），包裹 <span>。不引入外部依赖。
 */
export function highlightCode(source: string, language: string): string {
  const escaped = escapeHtml(source)
  const keywords = KEYWORDS[language] ?? []
  if (keywords.length === 0) {
    return escaped
  }
  // 组合正则：字符串 / 行注释 / 关键字。先转义所以引号是 &#39; &quot;
  // 但为可读性仍匹配原始引号（转义后源文本里不会出现裸引号，无需处理）。
  const kwPattern = keywords.map((k) => k.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')).join('|')
  const re = new RegExp(
    `(&quot;[^&]*&quot;|&#39;[^&]*&#39;)|(//[^\\n]*|--[^\\n]*|#[^\\n]*)|\\b(${kwPattern})\\b`,
    language === 'sql' ? 'gi' : 'g',
  )
  return escaped.replace(re, (match, str, comment, kw) => {
    if (str) return `<span class="tok-str">${str}</span>`
    if (comment) return `<span class="tok-comment">${comment}</span>`
    if (kw) return `<span class="tok-kw">${kw}</span>`
    return match
  })
}
