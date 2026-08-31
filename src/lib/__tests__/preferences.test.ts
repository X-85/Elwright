import { describe, expect, it, beforeEach, vi } from 'vitest'
import { nextTick } from 'vue'

import {
  mergePreferences,
  updatePreferences,
  saveLastView,
  resolveStartupView,
  preferences,
  initializePreferences,
} from '../preferences'

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

describe('preferences 副作用（localStorage + DOM）', () => {
  beforeEach(() => {
    localStorage.clear()
    document.documentElement.className = ''
    document.documentElement.style.cssText = ''
    preferences.value = mergePreferences(null)
  })

  it('updatePreferences 部分覆盖不影响其他字段', () => {
    updatePreferences({ density: 'compact', uiScale: 125 })
    expect(preferences.value.density).toBe('compact')
    expect(preferences.value.uiScale).toBe(125)
    expect(preferences.value.terminalFontSize).toBe(13)
  })

  it('initializePreferences 应用 density/zoom 并写 localStorage', async () => {
    updatePreferences({ density: 'compact', uiScale: 125 })
    initializePreferences()
    expect(document.documentElement.classList.contains('density-compact')).toBe(true)
    expect(document.documentElement.style.zoom).toBe('1.25')
    // 触发 watch 写存储（watch 默认 flush: pre，需 nextTick）
    updatePreferences({ density: 'comfortable' })
    await nextTick()
    const stored = JSON.parse(localStorage.getItem('elwright.preferences') ?? 'null')
    expect(stored?.density).toBe('comfortable')
  })

  it('initializePreferences 在 uiScale=100 时清除 zoom', async () => {
    updatePreferences({ uiScale: 110 })
    initializePreferences()
    await nextTick()
    expect(document.documentElement.style.zoom).toBe('1.1')
    updatePreferences({ uiScale: 100 })
    await nextTick()
    // jsdom 把 setProperty('zoom', '') 解析为 'normal'；只验证不再是 '1.1'
    expect(document.documentElement.style.zoom).not.toBe('1.1')
  })

  it('initializePreferences localStorage 写入失败不抛', () => {
    const setItemSpy = vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
      throw new Error('quota')
    })
    initializePreferences()
    updatePreferences({ density: 'compact' })
    setItemSpy.mockRestore()
  })
})

describe('saveLastView / resolveStartupView', () => {
  beforeEach(() => {
    localStorage.clear()
  })

  it('saveLastView 只接受 STARTUP_VIEW_OPTIONS 中非 last 的视图', () => {
    saveLastView('code')
    expect(localStorage.getItem('elwright.last-view')).toBe('code')
    saveLastView('last') // 'last' 不在合法集合
    expect(localStorage.getItem('elwright.last-view')).toBe('code') // 未被覆盖
    saveLastView('bogus')
    expect(localStorage.getItem('elwright.last-view')).toBe('code')
  })

  it('resolveStartupView: last + 无记录 → toolbox', () => {
    expect(resolveStartupView('last')).toBe('toolbox')
  })

  it('resolveStartupView: last + 有记录 → 上次视图', () => {
    saveLastView('chat')
    expect(resolveStartupView('last')).toBe('chat')
  })

  it('resolveStartupView: last + 非法记录 → toolbox', () => {
    localStorage.setItem('elwright.last-view', 'bogus')
    expect(resolveStartupView('last')).toBe('toolbox')
  })

  it('resolveStartupView: 固定值直返', () => {
    expect(resolveStartupView('code')).toBe('code')
  })
})
