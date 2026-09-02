import { describe, expect, it } from 'vitest'

import { escapeHtml, highlightCode } from '../codeHighlight'

describe('escapeHtml', () => {
  it('转义 HTML 特殊字符', () => {
    expect(escapeHtml('<script>alert("x")</script>')).toBe(
      '&lt;script&gt;alert(&quot;x&quot;)&lt;/script&gt;',
    )
  })
})

describe('highlightCode', () => {
  it('先转义再着色：恶意内容不会产生裸 HTML', () => {
    const out = highlightCode('<img src=x onerror=alert(1)>', 'java')
    expect(out).not.toContain('<img')
    expect(out).toContain('&lt;img')
  })

  it('java 关键词、字符串、行注释分别着色', () => {
    const out = highlightCode('public class A { String s = "hi"; } // note\n', 'java')
    expect(out).toContain('<span class="tok-kw">public</span>')
    expect(out).toContain('<span class="tok-kw">class</span>')
    expect(out).toContain('<span class="tok-str">&quot;hi&quot;</span>')
    expect(out).toContain('<span class="tok-comment">// note</span>')
  })

  it('不认识的语言原样转义返回', () => {
    const out = highlightCode('plain <text>', 'unknown-lang')
    expect(out).toBe('plain &lt;text&gt;')
  })

  it('关键词匹配有词边界：users 不应命中 user', () => {
    const out = highlightCode('users.forEach(', 'java')
    expect(out).not.toContain('tok-kw">user</span>')
  })
})
