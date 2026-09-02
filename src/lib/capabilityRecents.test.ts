import { describe, expect, it } from 'vitest'
import {
  MAX_RECENTS,
  loadFavorites,
  loadRecents,
  recordRecent,
  toggleFavorite,
} from './capabilityRecents'

function clearKeys() {
  localStorage.removeItem('elwright-capability-favorites')
  localStorage.removeItem('elwright-capability-recents')
}

describe('capabilityRecents', () => {
  it('空存储返回默认空列表', () => {
    clearKeys()
    expect(loadFavorites()).toEqual([])
    expect(loadRecents()).toEqual([])
  })

  it('toggleFavorite 去重切换并持久化', () => {
    clearKeys()
    expect(toggleFavorite('a')).toEqual(['a'])
    expect(toggleFavorite('b')).toEqual(['b', 'a'])
    expect(toggleFavorite('a')).toEqual(['b'])
    expect(loadFavorites()).toEqual(['b'])
    clearKeys()
  })

  it('recordRecent 去重置顶并淘汰最旧', () => {
    clearKeys()
    for (let i = 0; i < MAX_RECENTS + 2; i++) {
      recordRecent(`cap-${i}`)
    }
    const recents = loadRecents()
    expect(recents).toHaveLength(MAX_RECENTS)
    expect(recents[0].id).toBe(`cap-${MAX_RECENTS + 1}`)
    expect(recents.some((r) => r.id === 'cap-0')).toBe(false)

    // 重复使用：置顶不新增
    recordRecent(`cap-${MAX_RECENTS + 1}`)
    expect(loadRecents()).toHaveLength(MAX_RECENTS)
    clearKeys()
  })

  it('损坏的存储内容回退默认值', () => {
    localStorage.setItem('elwright-capability-recents', '{broken')
    expect(loadRecents()).toEqual([])
    localStorage.setItem('elwright-capability-favorites', '"not-array"')
    expect(loadFavorites()).toEqual([])
    clearKeys()
  })
})
