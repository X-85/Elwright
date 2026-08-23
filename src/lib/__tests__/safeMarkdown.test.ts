import { describe, expect, it } from 'vitest'
import { renderChatMarkdown } from '../safeMarkdown'

// ADR-002：AI 输出按不可信文本渲染。这里锁安全底线与常用渲染行为。

describe('renderChatMarkdown · 安全底线', () => {
  it('原始 HTML 标签被转义为文本，不成为标签', () => {
    const html = renderChatMarkdown('<script>alert(1)</script>')
    expect(html).not.toContain('<script>')
    expect(html).toContain('&lt;script&gt;')
  })

  it('img onerror 注入被转义', () => {
    const html = renderChatMarkdown('<img src=x onerror=alert(1)>')
    expect(html).not.toMatch(/<img/)
    expect(html).toContain('&lt;img')
  })

  it('javascript: 链接降级为纯文本（不产生 <a>）', () => {
    const html = renderChatMarkdown('[点我](javascript:alert(1))')
    expect(html).not.toMatch(/<a\s/)
    expect(html).toContain('点我')
  })

  it('data: 协议链接同样被拒', () => {
    const html = renderChatMarkdown('[x](data:text/html,<b>)')
    expect(html).not.toMatch(/<a\s/)
  })

  it('href 属性逃逸（引号/空白）被拒：恶意片段不进入任何标签属性', () => {
    const html = renderChatMarkdown('[x](https://a.com/" onmouseover="alert(1))')
    // 防的是属性逃逸：onmouseover 绝不能以裸引号形式进入标签（marked 会把
    // 裸 URL 段 autolink，引号文本被转义为 &quot; 实体，无逃逸）
    expect(html).not.toMatch(/onmouseover="\w/)
    expect(html).toContain('&quot; onmouseover=&quot;')
  })

  it('恶意图片协议被拒，降级为 alt 文本', () => {
    const html = renderChatMarkdown('![alt](javascript:alert(1))')
    expect(html).not.toMatch(/<img/)
  })
})

describe('renderChatMarkdown · 正常渲染', () => {
  it('http(s)/mailto/锚点/相对路径链接放行且带 noopener', () => {
    const html = renderChatMarkdown('[a](https://a.com) [b](mailto:x@y.z) [c](#sec) [d](./rel)')
    const links = html.match(/<a [^>]*>/g) ?? []
    expect(links).toHaveLength(4)
    expect(html).toContain('rel="noopener noreferrer"')
  })

  it('代码块转义内容并带复制按钮外层', () => {
    const html = renderChatMarkdown('```rust\nlet x = "<b>";\n```')
    expect(html).toContain('class="code-block"')
    expect(html).toContain('code-copy-btn')
    expect(html).toContain('&lt;b&gt;')
  })

  it('行内代码同样转义', () => {
    const html = renderChatMarkdown('use `<i>` tags')
    expect(html).toContain('<code>&lt;i&gt;</code>')
  })

  it('普通加粗/标题不受影响', () => {
    const html = renderChatMarkdown('# 标题\n\n**加粗**')
    expect(html).toContain('<h1>标题</h1>')
    expect(html).toContain('<strong>加粗</strong>')
  })
})
