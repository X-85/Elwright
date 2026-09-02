// 渐进式发布成长体系（ADR-001）：解锁规则与进度的纯函数收口。
// 语义沿用 MVP：unlockAfterUses = 累计使用「任意能力」的本地次数门槛。

import type { Capability } from './bridge'

/** 是否已解锁：tier ≤ 1 恒解锁；tier > 1 需累计使用达 unlockAfterUses（缺省视为未开放条件）。 */
export function isUnlocked(cap: Capability, totalUses: number): boolean {
  const tier = cap.releaseTier ?? 1
  if (tier <= 1) return true
  const required = cap.unlockAfterUses
  return required !== undefined && totalUses >= required
}

export interface LockedInfo {
  id: string
  name: string
  /** 解锁门槛（累计使用次数）；undefined = 未开放解锁条件 */
  threshold?: number
  /** 距解锁还差的使用次数；threshold 缺省时为 undefined */
  remaining?: number
}

export interface GrowthSummary {
  /** 全部能力总数（用于展示「T 次累计」上下文的调用方自行取用 totalUses） */
  locked: LockedInfo[]
  /** 距离最近的可解锁项（threshold 已知且剩余最少）；无可进度项时为 null */
  nearest: { name: string; threshold: number; remaining: number } | null
}

/** 汇总当前锁定项与最近的解锁进度。 */
export function growthSummary(capabilities: Capability[], totalUses: number): GrowthSummary {
  const locked: LockedInfo[] = capabilities
    .filter((c) => !isUnlocked(c, totalUses))
    .map((c) => {
      const threshold = c.unlockAfterUses
      return {
        id: c.id,
        name: c.name,
        threshold,
        remaining: threshold !== undefined ? Math.max(0, threshold - totalUses) : undefined,
      }
    })
  const withProgress = locked.filter(
    (l): l is LockedInfo & { threshold: number; remaining: number } =>
      l.threshold !== undefined && l.remaining !== undefined,
  )
  const nearest = withProgress.length
    ? (({ name, threshold, remaining }) => ({ name, threshold, remaining }))(
        withProgress.reduce((a, b) => (b.remaining < a.remaining ? b : a)),
      )
    : null
  return { locked, nearest }
}

/** [prev, curr] 区间内跨过解锁门槛的能力名（用于解锁时刻的 toast）。 */
export function newlyUnlocked(capabilities: Capability[], prev: number, curr: number): string[] {
  if (curr <= prev) return []
  return capabilities
    .filter((c) => {
      const required = c.unlockAfterUses
      if (required === undefined) return false
      const tier = c.releaseTier ?? 1
      return tier > 1 && prev < required && curr >= required
    })
    .map((c) => c.name)
}
