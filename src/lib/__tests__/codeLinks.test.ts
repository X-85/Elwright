import { describe, expect, it } from 'vitest'

import { codeLinkMarker, parseCodeLinks } from '../codeLinks'

describe('parseCodeLinks', () => {
  it('拆出代码位置标记与普通文本', () => {
    const parts = parseCodeLinks('代码 `/a/b.java:42` review 用')
    expect(parts).toHaveLength(3)
    expect(parts[1]).toEqual({ kind: 'code', text: '`/a/b.java:42`', path: '/a/b.java', line: 42 })
    expect(parts[0].kind).toBe('text')
    expect(parts[2].text).toBe(' review 用')
  })

  it('无标记时原样返回', () => {
    expect(parseCodeLinks('普通 todo')).toEqual([{ kind: 'text', text: '普通 todo' }])
  })

  it('行号必须是数字才解析', () => {
    const parts = parseCodeLinks('`/a/b.java:abc`')
    expect(parts).toHaveLength(1)
    expect(parts[0].kind).toBe('text')
  })
})

describe('codeLinkMarker', () => {
  it('生成反引号包裹的标记', () => {
    expect(codeLinkMarker('/a/b.java', 7)).toBe('`/a/b.java:7`')
  })
})
