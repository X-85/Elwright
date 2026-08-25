import { beforeEach, describe, expect, it } from 'vitest'
import { createBridge } from '../bridge'

// browserBridge 的工作工具栏模拟存储（进程内，刷新即失口径）：
// UI 完整体验走它，持久化只在桌面壳。这里锁行为语义与 Rust core 一致。

// jsdom 无 __TAURI_INTERNALS__ → createBridge 必须落在 browser 实现
const bridge = createBridge()

describe('browserBridge 工作工具栏模拟存储', () => {
  beforeEach(() => {
    // 清场：删掉本套件前面测试留下的条目（模块级存储跨测试共享）
    void bridge.todoList().then((items) => {
      for (const t of items) void bridge.todoRemove(t.id)
    })
  })

  it('Todo 往返：add → list → toggle → remove', async () => {
    const a = await bridge.todoAdd('写周报')
    expect(a.done).toBe(false)
    expect(a.completedAt).toBeNull()
    expect(a.id).toBeGreaterThanOrEqual(0)

    const b = await bridge.todoAdd('复查 PR')
    expect(b.id).not.toBe(a.id)

    const toggled = await bridge.todoToggle(b.id)
    expect(toggled.done).toBe(true)
    expect(toggled.completedAt).toBeTypeOf('string')

    const back = await bridge.todoToggle(b.id)
    expect(back.done).toBe(false)
    expect(back.completedAt).toBeNull()

    let list = await bridge.todoList()
    expect(list).toHaveLength(2)

    await bridge.todoRemove(a.id)
    list = await bridge.todoList()
    expect(list).toHaveLength(1)
    expect(list[0]!.id).toBe(b.id)
  })

  it('未知 id：toggle/remove 抛中文错误', async () => {
    await expect(bridge.todoToggle(99999)).rejects.toThrow(/不存在/)
    await expect(bridge.todoRemove(99999)).rejects.toThrow(/不存在/)
  })

  it('Note 往返：save → get → list 倒序', async () => {
    expect(await bridge.noteGet('2099-01-02')).toBeNull()

    await bridge.noteSave('2099-01-02', '# 今日')
    await bridge.noteSave('2099-01-01', '昨天')
    expect(await bridge.noteGet('2099-01-02')).toBe('# 今日')

    const dates = await bridge.noteList()
    expect(dates[0]).toBe('2099-01-02')
    expect(dates).toContain('2099-01-01')
  })

  it('非法日期：save 抛中文错误（与 core 校验同口径）', async () => {
    await expect(bridge.noteSave('../evil', 'x')).rejects.toThrow(/日期/)
    await expect(bridge.noteSave('2099-1-2', 'x')).rejects.toThrow(/日期/)
  })
})
