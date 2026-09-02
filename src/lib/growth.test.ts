import { describe, expect, it } from 'vitest'
import type { Capability } from './bridge'
import { growthSummary, isUnlocked, newlyUnlocked } from './growth'

function cap(partial: Partial<Capability>): Capability {
  return {
    id: 'test-cap',
    name: '测试能力',
    type: 'script',
    origin: 'builtin',
    ...partial,
  } as Capability
}

describe('isUnlocked', () => {
  it('tier 1 恒解锁', () => {
    expect(isUnlocked(cap({ releaseTier: 1 }), 0)).toBe(true)
  })

  it('tier 2 达门槛解锁，未达门槛锁定', () => {
    const c = cap({ releaseTier: 2, unlockAfterUses: 3 })
    expect(isUnlocked(c, 2)).toBe(false)
    expect(isUnlocked(c, 3)).toBe(true)
  })

  it('tier 2 缺 unlockAfterUses 视为未开放条件（恒锁）', () => {
    expect(isUnlocked(cap({ releaseTier: 2 }), 999)).toBe(false)
  })
})

describe('growthSummary', () => {
  it('无锁定项时 nearest 为 null', () => {
    const s = growthSummary([cap({ releaseTier: 1 })], 0)
    expect(s.locked).toHaveLength(0)
    expect(s.nearest).toBeNull()
  })

  it('nearest 取剩余最少且带门槛的项，缺门槛项不参与', () => {
    const caps = [
      cap({ id: 'a', name: '甲', releaseTier: 2, unlockAfterUses: 5 }),
      cap({ id: 'b', name: '乙', releaseTier: 2, unlockAfterUses: 2 }),
      cap({ id: 'c', name: '丙', releaseTier: 2 }),
    ]
    const s = growthSummary(caps, 1)
    expect(s.locked).toHaveLength(3)
    expect(s.nearest).toEqual({ name: '乙', threshold: 2, remaining: 1 })
    // 缺门槛项 remaining 为 undefined
    expect(s.locked.find((l) => l.id === 'c')?.remaining).toBeUndefined()
  })
})

describe('newlyUnlocked', () => {
  it('跨阈值时返回能力名，未跨过/倒退不返回', () => {
    const caps = [cap({ id: 'x', name: '周报生成', releaseTier: 2, unlockAfterUses: 3 })]
    expect(newlyUnlocked(caps, 2, 3)).toEqual(['周报生成'])
    expect(newlyUnlocked(caps, 1, 2)).toEqual([])
    expect(newlyUnlocked(caps, 5, 3)).toEqual([])
    // tier 1 无门槛项永不触发
    expect(newlyUnlocked([cap({ releaseTier: 1 })], 0, 9)).toEqual([])
  })
})
