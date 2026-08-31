import { describe, expect, it } from 'vitest'
import { extractFirstDiff, renderDiffLines } from '../patch'

describe('extractFirstDiff', () => {
  it('识别带 ```diff 围栏的合法 diff', () => {
    const text = [
      '下面是补丁：',
      '```diff',
      '--- a/src/foo.rs',
      '+++ b/src/foo.rs',
      '@@ -1,3 +1,3 @@',
      ' alpha',
      '-beta',
      '+BETA',
      ' gamma',
      '```',
      '请确认。',
    ].join('\n')
    const out = extractFirstDiff(text)
    expect(out).not.toBeNull()
    expect(out).toContain('--- a/src/foo.rs')
    expect(out).toContain('+++ b/src/foo.rs')
    expect(out).toContain('-beta')
  })

  it('未含 diff 头返回 null', () => {
    const text = '```\nplain code\n```'
    expect(extractFirstDiff(text)).toBeNull()
  })

  it('缺少 ``` 围栏但含 diff 头不应识别（边界由后端兜底）', () => {
    const text = '--- a/x\n+++ b/x\n@@\n-old\n+new\n'
    expect(extractFirstDiff(text)).toBeNull()
  })
})

describe('renderDiffLines', () => {
  it('渲染 add/del/ctx 三类行', () => {
    const lines = renderDiffLines('--- a/x\n+++ b/x\n@@ -1 +1 @@\n-old\n+new\n kept')
    expect(lines).toEqual([
      { kind: 'del', text: 'old' },
      { kind: 'add', text: 'new' },
      { kind: 'ctx', text: 'kept' },
    ])
  })
})