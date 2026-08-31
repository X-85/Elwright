/**
 * Q19 设置中心：模型档案名校验（前端校验 + 后端权威）
 * 规则：仅小写字母/数字/-/_，长度 1-32。前端先粗筛，后端走权威归一化。
 */

export function validateProfileName(
  name: string,
  existing: string[],
): string {
  const trimmed = name.trim()
  if (!trimmed) return '档案名不能为空'
  if (trimmed.length > 32) return '档案名长度 1-32'
  if (!/^[a-zA-Z0-9_-]+$/.test(trimmed)) {
    return '档案名仅允许字母/数字/-/_'
  }
  const lower = trimmed.toLowerCase()
  if (existing.includes(lower)) return '档案名已存在'
  return ''
}

export function normalizeProfileName(name: string): string {
  return name.trim().toLowerCase()
}