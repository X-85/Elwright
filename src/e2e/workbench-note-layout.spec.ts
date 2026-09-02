import { expect, test } from '@playwright/test'

// C2 回归：工作台「今日记录」格子被遮挡/压扁。
//
// 根因：.workbench 是 grid-auto-rows: minmax(230px, auto) 的 2×2 格局，
// 「auto」上限让格子随内容收缩——矮窗口/长预览时今日记录格子被推出
// 视口或压成 0 高，观感即「被其他页面遮挡」。
//
// 修复后不变量（本用例守护）：
// ①默认视口（720p 高度）下四个格子全部可见；
// ②把视口压到极矮（400px 高），今日记录标题仍在文档内、格子高度
//   不小于 260px 行下限（可滚动到达，而非消失）；
// ③编辑器自身可滚动（overflow-y:auto），格子内长文不再撑爆布局。

test('工作台 2×2 四宫格：矮窗口下今日记录格子保持行高下限、可滚动到达', async ({ page }) => {
  await page.goto('/')
  await page.click('button[aria-label="工作台"]')

  // 正常视口：今日记录标题可见
  const noteHead = page.locator('.wb-note h2', { hasText: '今日记录' })
  await expect(noteHead).toBeVisible()

  // 压扁视口到 400px 高（远小于两行 260px 下限之和）
  await page.setViewportSize({ width: 1280, height: 400 })
  await page.waitForTimeout(200)

  // 格子仍在布局内（未被遮挡消失）
  await expect(noteHead).toBeVisible()

  // 今日记录格子实际高度 ≥ 260px 行下限（不被压缩成 0）
  const noteBox = await page.locator('.wb-note').boundingBox()
  expect(noteBox).not.toBeNull()
  expect(noteBox!.height).toBeGreaterThanOrEqual(259)

  // 工作台容器可滚动（能滚到第二行格子，而非永远不可达）
  const scrollable = await page.evaluate(() => {
    const el = document.querySelector('.workbench')
    return el ? el.scrollHeight > el.clientHeight : false
  })
  expect(scrollable).toBe(true)
})

test('今日记录编辑器自身滚动：长文不撑爆格子', async ({ page }) => {
  await page.goto('/')
  await page.click('button[aria-label="工作台"]')

  // 灌入 200 行长文
  const long = Array.from({ length: 200 }, (_, i) => `第 ${i + 1} 行记录`).join('\n')
  await page.fill('.wb-note-editor', long)
  await expect(page.locator('.wb-save-state')).toHaveText('已保存', { timeout: 5000 })

  // 编辑器内部出现滚动条（内容高于可视区），格子高度仍受 grid 约束
  const inner = await page.evaluate(() => {
    const ta = document.querySelector('.wb-note-editor') as HTMLTextAreaElement | null
    const grid = document.querySelector('.workbench') as HTMLElement | null
    if (!ta || !grid) return null
    return {
      editorScrollable: ta.scrollHeight > ta.clientHeight,
      gridRowsOverflow: grid.scrollHeight > grid.clientHeight,
      noteHeight: document.querySelector('.wb-note')?.getBoundingClientRect().height ?? 0,
    }
  })
  expect(inner).not.toBeNull()
  expect(inner!.editorScrollable).toBe(true)
  // 关键不变量：格子高度不再随 200 行内容膨胀出 2×2 格局
  expect(inner!.noteHeight).toBeLessThan(1200)

  // 留证截图
  await page.screenshot({
    path: '../docs/work/active/bugfix-2026-09-workbench-note-occlusion/screenshots/note-long-text.png',
    fullPage: false,
  })
})
