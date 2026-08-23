import { describe, expect, it } from 'vitest'
import { compareVersions } from '../bridge'

// 语义须与 core::version::is_newer 一致（Rust 侧有单测）：
// 逐段数值比较、段内非数字后缀取前导数字、缺段视为 0。
// 这里锁前端行为：检查更新按钮的「有新版本」判定依赖它。

describe('compareVersions', () => {
  it('常规三段比较：更新返回正数', () => {
    expect(compareVersions('0.2.0', '0.1.4')).toBeGreaterThan(0)
    expect(compareVersions('1.0.0', '0.9.9')).toBeGreaterThan(0)
  })

  it('相同版本返回 0', () => {
    expect(compareVersions('0.1.4', '0.1.4')).toBe(0)
  })

  it('旧版本返回负数', () => {
    expect(compareVersions('0.1.0', '0.1.4')).toBeLessThan(0)
  })

  it('v 前缀剥离（tag 形态 vs 配置形态）', () => {
    expect(compareVersions('v0.1.4', '0.1.4')).toBe(0)
    expect(compareVersions('v0.2.0', 'v0.1.4')).toBeGreaterThan(0)
  })

  it('缺段视为 0：0.1 == 0.1.0', () => {
    expect(compareVersions('0.1', '0.1.0')).toBe(0)
    expect(compareVersions('0.1.1', '0.1')).toBeGreaterThan(0)
  })

  it('非数字段取前导数字（0.1.0-beta.1 之类不崩）', () => {
    // beta 解析为 0：0.1.0-beta.1 ≈ 0.1.0.0.1 —— 与 0.1.0 相比更新（后缀容忍）
    expect(compareVersions('0.1.0-beta.1', '0.1.0')).not.toBeNaN()
  })

  it('跨数量级正确（1.10 > 1.9）', () => {
    expect(compareVersions('1.10.0', '1.9.0')).toBeGreaterThan(0)
  })
})
