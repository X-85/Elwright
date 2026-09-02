// 工作台「常用能力」：收藏与最近使用的本地存储（工作台 ADR-001）。
// 全部为前端本机数据（localStorage），读写失败静默回退默认值（离网友好）。

const FAVORITES_KEY = 'elwright-capability-favorites'
const RECENTS_KEY = 'elwright-capability-recents'

/** 最近使用上限（超出淘汰最旧）。 */
export const MAX_RECENTS = 8

export interface RecentUse {
  id: string
  /** 最后一次使用的 epoch 毫秒 */
  at: number
}

function readJson<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(key)
    if (!raw) return fallback
    return JSON.parse(raw) as T
  } catch {
    return fallback
  }
}

function writeJson(key: string, value: unknown): void {
  try {
    localStorage.setItem(key, JSON.stringify(value))
  } catch {
    // 存储不可用（隐私模式/配额）——静默忽略，功能降级为会话内状态
  }
}

export function loadFavorites(): string[] {
  const raw = readJson<string[]>(FAVORITES_KEY, [])
  return Array.isArray(raw) ? raw.filter((id) => typeof id === 'string') : []
}

export function saveFavorites(ids: string[]): void {
  writeJson(FAVORITES_KEY, ids)
}

/** 切换收藏；返回新列表。 */
export function toggleFavorite(id: string): string[] {
  const ids = loadFavorites()
  const next = ids.includes(id) ? ids.filter((x) => x !== id) : [id, ...ids]
  saveFavorites(next)
  return next
}

export function loadRecents(): RecentUse[] {
  const raw = readJson<RecentUse[]>(RECENTS_KEY, [])
  if (!Array.isArray(raw)) return []
  return raw.filter(
    (r) => r && typeof r.id === 'string' && typeof r.at === 'number',
  )
}

/** 记录一次使用：去重置顶、淘汰最旧、落盘。返回新列表。 */
export function recordRecent(id: string): RecentUse[] {
  const rest = loadRecents().filter((r) => r.id !== id)
  const next = [{ id, at: Date.now() }, ...rest].slice(0, MAX_RECENTS)
  writeJson(RECENTS_KEY, next)
  return next
}
