import { describe, expect, it } from 'vitest'

import { mergePreferences } from '../preferences'

describe('mergePreferences', () => {
  it('非对象输入回退全默认', () => {
    expect(mergePreferences(null)).toEqual(mergePreferences(undefined))
    const d = mergePreferences(null)
    expect(d.startupView).toBe('last')
    expect(d.terminalFontSize).toBe(13)
    expect(d.uiScale).toBe(100)
  })

  it('合法字段覆盖、非法字段回退默认', () => {
    const out = mergePreferences({
      startupView: 'code',
      autoUpdateCheck: true,
      density: 'compact',
      uiScale: 125,
      terminalFontSize: 99,
      terminalScrollback: 12345,
      terminalFontFamily: '   ',
    })
    expect(out.startupView).toBe('code')
    expect(out.autoUpdateCheck).toBe(true)
    expect(out.density).toBe('compact')
    expect(out.uiScale).toBe(125)
    expect(out.terminalFontSize).toBe(13, '非法字号回退默认')
    expect(out.terminalScrollback).toBe(10000, '非法滚动历史回退默认')
    expect(out.terminalFontFamily).not.toBe('   ')
  })
})
