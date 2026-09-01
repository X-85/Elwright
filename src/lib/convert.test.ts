import { describe, expect, it } from 'vitest'
import {
  base64Decode,
  base64Encode,
  dateToTimestamp,
  formatJson,
  minifyJson,
  timestampToDate,
} from './convert'

describe('JSON', () => {
  it('格式化与压缩互逆', () => {
    const raw = '{"a":1,"b":[2,3]}'
    const pretty = formatJson(raw)
    expect(pretty).toContain('\n  "a": 1')
    expect(minifyJson(pretty)).toBe(raw)
  })

  it('非法/空输入抛中文错误', () => {
    expect(() => formatJson('   ')).toThrow('输入为空')
    expect(() => formatJson('{broken')).toThrow('JSON 解析失败')
  })
})

describe('Base64', () => {
  it('UTF-8（中文）编解码往返', () => {
    const text = '中文 abc 🎉'
    expect(base64Decode(base64Encode(text))).toBe(text)
  })

  it('解码失败抛中文错误', () => {
    expect(() => base64Decode('!!!not-base64!!!')).toThrow('Base64 解码失败')
  })
})

describe('时间戳 ⇄ 日期', () => {
  it('秒级与毫秒级自动识别', () => {
    // 2026-01-02 03:04:05 UTC = 1767325445 s = 1767325445000 ms
    expect(timestampToDate('1767325445')).toMatch(/^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$/)
    expect(timestampToDate('1767325445000')).toBe(timestampToDate('1767325445'))
  })

  it('非法输入抛中文错误', () => {
    expect(() => timestampToDate('abc')).toThrow('10 位')
    expect(() => timestampToDate('')).toThrow('10 位')
  })

  it('日期 → 毫秒时间戳往返', () => {
    const ts = dateToTimestamp('2026-01-02 03:04:05')
    expect(ts).toBe(String(new Date('2026-01-02T03:04:05').getTime()))
    expect(timestampToDate(ts)).toMatch(/^\d{4}-\d{2}-\d{2} 03:04:05$/)
  })
})
