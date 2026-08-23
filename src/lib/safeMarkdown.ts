import { Marked } from 'marked'

/**
 * 不可信 Markdown 渲染（AI 对话用，见 docs/features/chat/decisions/ADR-002）。
 *
 * 模型输出按不可信文本处理：
 * - 原始 HTML（块级/行内标签）原样转义为文本，不执行、不渲染为标签；
 * - link/image 的 href 仅放行 http(s)/mailto/锚点/相对路径，且不允许
 *   href 内出现引号/空白（防属性逃逸）——`javascript:` 等协议链接降级为纯文本；
 * - 代码块/行内代码不受影响：marked 默认 renderer 自行转义其内容。
 *
 * 与 CapabilityDetail 的 trustedMarkdown 区别：那里渲染本地 resources/ 可信文件，
 * 直出 v-html；这里多一层 renderer 收敛，零新依赖。
 */

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

const SAFE_HREF = /^(https?:|mailto:|#|(?:\/|\.{1,2}\/))/i

/** href 白名单校验：协议限定 + 值内不允许引号/空白/尖括号。 */
function safeHref(href: string | null | undefined): string | null {
  if (!href) return null
  const trimmed = href.trim()
  if (/[\s"'<>]/.test(trimmed)) return null
  return SAFE_HREF.test(trimmed) ? trimmed : null
}

const chatMarked = new Marked({
  renderer: {
    // 覆写默认 code renderer：加复制按钮外层（点击委托由 ChatView 处理）
    code({ text, lang }) {
      const langAttr = lang ? ` class="language-${escapeHtml(lang)}"` : ''
      return `<div class="code-block"><button class="code-copy-btn" type="button">复制</button><pre><code${langAttr}>${escapeHtml(text)}</code></pre></div>`
    },
    html({ text }) {
      return escapeHtml(text)
    },
    link({ href, title, tokens }) {
      const url = safeHref(href)
      if (!url) return this.parser.parseInline(tokens)
      const titleAttr = title ? ` title="${escapeHtml(title)}"` : ''
      return `<a href="${url}" target="_blank" rel="noopener noreferrer"${titleAttr}>${this.parser.parseInline(tokens)}</a>`
    },
    image({ href, title, text }) {
      const url = safeHref(href)
      if (!url) return escapeHtml(text ?? '')
      const alt = text ? ` alt="${escapeHtml(text)}"` : ''
      const titleAttr = title ? ` title="${escapeHtml(title)}"` : ''
      return `<img src="${url}"${alt}${titleAttr}>`
    },
  },
})

/** 渲染不可信 Markdown 为安全 HTML（同步）。 */
export function renderChatMarkdown(text: string): string {
  return chatMarked.parse(text, { async: false }) as string
}
