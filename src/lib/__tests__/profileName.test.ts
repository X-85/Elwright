import { describe, expect, it } from 'vitest'
import { normalizeProfileName, validateProfileName } from '../profileName'

describe('validateProfileName', () => {
  it('接受合法档案名（小写/数字/-/_）', () => {
    expect(validateProfileName('default', [])).toBe('')
    expect(validateProfileName('local-ollama', [])).toBe('')
    expect(validateProfileName('work_2', [])).toBe('')
    expect(validateProfileName('a', [])).toBe('')
  })

  it('拒绝空名 / 超长 / 非法字符', () => {
    expect(validateProfileName('', [])).toBe('档案名不能为空')
    expect(validateProfileName('   ', [])).toBe('档案名不能为空')
    expect(validateProfileName('a'.repeat(33), [])).toBe('档案名长度 1-32')
    expect(validateProfileName('has space', [])).toBe('档案名仅允许字母/数字/-/_')
    expect(validateProfileName('中文', [])).toBe('档案名仅允许字母/数字/-/_')
  })

  it('大小写不敏感 + 已有检查', () => {
    expect(validateProfileName('Work', ['work'])).toBe('档案名已存在')
    expect(validateProfileName('NEW', ['new'])).toBe('档案名已存在')
    expect(validateProfileName('fresh', ['work'])).toBe('')
  })
})

describe('normalizeProfileName', () => {
  it('trim + lowercase', () => {
    expect(normalizeProfileName('  Work  ')).toBe('work')
    expect(normalizeProfileName('Default')).toBe('default')
    expect(normalizeProfileName('local-ollama')).toBe('local-ollama')
  })
})