import { describe, expect, it } from 'vitest'
import { dictKeysInSync, locale, setLocale, t } from './i18n'

describe('i18n', () => {
  it('默认 zh-CN；t 返回中文', () => {
    expect(locale.value).toBe('zh-CN')
    expect(t('settings.title')).toBe('设置')
  })

  it('切语言后 t 返回对应译文', () => {
    setLocale('en')
    expect(t('settings.title')).toBe('Settings')
    setLocale('zh-CN')
    expect(t('settings.title')).toBe('设置')
  })

  it('缺 key 回退 zh-CN，再回退 key 本身', () => {
    setLocale('en')
    expect(t('settings.nonexistent-key')).toBe('settings.nonexistent-key')
    setLocale('zh-CN')
  })

  it('双语字典键集完全一致（完整性守卫）', () => {
    expect(dictKeysInSync()).toBe(true)
  })
})
