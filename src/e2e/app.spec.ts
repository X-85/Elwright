import { expect, test } from '@playwright/test'

// 黑盒浏览器级冒烟：走 vite dev 的只读端点（/api/capabilities、/api/file），
// 覆盖 browserBridge ↔ dev 插件接缝与预览模式降级守卫。
// 真实 IPC（桌面壳）由 src-tauri/tests/terminal_ipc.rs 覆盖，两层互补。

test.beforeEach(async ({ page }) => {
  await page.goto('/')
  // 等首屏能力加载完（列表渲染出条目）再断言，避免竞态
  await expect(page.locator('.cap-item').first()).toBeVisible()
})

test('工具箱默认核心视图 3 项，查看全部后 4 项（weekly-report 进阶档位，ADR-001）', async ({ page }) => {
  // 默认核心视图：进阶档位隐藏，提示行显示最近解锁进度
  await expect(page.locator('.cap-item')).toHaveCount(3)
  await expect(page.locator('.count')).toHaveText('3 / 4 项')
  await expect(page.locator('.growth-hint')).toContainText('距解锁「周报生成」还差 3 次')
  await expect(page.locator('.bridge-badge')).toHaveText('预览模式 · 浏览器')

  // 查看全部：进阶项出现并带待解锁徽标
  await page.click('.growth-toggle')
  await expect(page.locator('.cap-item')).toHaveCount(4)
  await expect(page.locator('.count')).toHaveText('4 / 4 项')
  await expect(page.locator('.cap-item:has-text("周报生成") .locked-badge')).toHaveText('待解锁')
})

test('筛选与搜索联动：script 过滤只剩文本统计', async ({ page }) => {
  await page.click('.filters button:text-is("脚本型")')
  await expect(page.locator('.cap-item')).toHaveCount(1)
  await expect(page.locator('.cap-name').first()).toHaveText('文本统计')

  // 清过滤后搜索无结果 → 空态
  await page.click('.filters button:text-is("全部")')
  await page.fill('.search', '不存在的关键字xyz')
  await expect(page.locator('.cap-empty')).toBeVisible()
})

test('知识型详情读 doc（/api/file 接缝），脚本型运行有降级文案', async ({ page }) => {
  // 知识型：view 走 doc 字段，经 /api/file 读真实文件渲染 markdown
  await page.click('.cap-item:has-text("能力类型速览")')
  const detail = page.locator('.detail')
  await expect(detail).toBeVisible()
  await expect(detail.locator('.detail-head h2')).toHaveText('能力类型速览')
  await expect(detail.locator('.markdown')).toBeVisible()
  // 接缝断开时 view.ok=false，只会渲染 .error 而非 .markdown
  await expect(detail.locator('.error')).toHaveCount(0)

  // 脚本型：浏览器无法 spawn 进程，运行按钮给出明确降级文案而非报错
  await page.click('.cap-item:has-text("文本统计")')
  await expect(detail.locator('.detail-head h2')).toHaveText('文本统计')
  await detail.locator('button.primary').click()
  await expect(detail.locator('.output')).toContainText('【预览模式】')
})

test('降级守卫：浏览器下终端按钮不渲染', async ({ page }) => {
  await expect(page.locator('button[title="打开或收起终端"]')).toHaveCount(0)
})

test('降级守卫：AI 对话页显示预览模式提示', async ({ page }) => {
  await page.click('button[aria-label="AI 对话"]')
  const note = page.locator('.chat-preview-note')
  await expect(note).toBeVisible()
  await expect(note).toContainText('【预览模式】')
})


test('降级守卫：消息页在浏览器预览不渲染邀请/添加按钮且本地会话可用', async ({ page }) => {
  await page.click('button[aria-label="消息会话"]')
  // 传输类按钮仅桌面渲染（getIdentity 失败 → desktop=false）
  await expect(page.locator('button[aria-label="邀请对方添加我"]')).toHaveCount(0)
  await expect(page.locator('button[aria-label="通过邀请添加联系人"]')).toHaveCount(0)
  // 本地会话创建仍可用
  await page.click('button[aria-label="新建会话"]')
  await page.fill('#peer-name', '李明')
  await page.click('button[type="submit"]')
  await expect(page.locator('.people-chat-head h3')).toHaveText('李明')
  await expect(page.locator('.local-status')).toContainText('本地会话')
})

test('工作台：Todo 添加/勾选/删除与今日记录自动保存（预览模式模拟存储）', async ({ page }) => {
  await page.click('button[aria-label="工作台"]')

  // Todo 添加 → 出现 → 计数
  await page.fill('.wb-input', 'e2e 待办事项')
  await page.click('.wb-add button[type="submit"]')
  const item = page.locator('.todo-item', { hasText: 'e2e 待办事项' })
  await expect(item).toBeVisible()
  await expect(item.locator('.todo-text')).toHaveText('e2e 待办事项')

  // 勾选 → 划线样式 + 计数 1/1
  await item.locator('input[type="checkbox"]').check()
  await expect(item).toHaveClass(/done/)
  await expect(page.locator('.wb-todo .wb-count')).toHaveText('1 / 1 完成')

  // 删除 → 消失 → 空态
  await item.locator('.todo-del').click()
  await expect(page.locator('.wb-empty').first()).toBeVisible()

  // 今日记录：输入 → 防抖后「已保存」徽标（模拟存储）
  await page.fill('.wb-note-editor', '# e2e 记录\n- 内容')
  await expect(page.locator('.wb-save-state')).toHaveText('已保存', { timeout: 5000 })

  // 预览切换：markdown 渲染出标题
  await page.click('.wb-preview-toggle')
  await expect(page.locator('.wb-note-preview h1')).toHaveText('e2e 记录')

  // 预览模式口径提示在
  await expect(page.locator('.wb-preview-note')).toContainText('【预览模式】')

  // 日期翻页：切前一天编辑器清空回空态
  await page.click('.wb-preview-toggle') // 回编辑态
  await page.click('button[aria-label="前一天"]')
  await expect(page.locator('.wb-note-editor')).toHaveValue('')
})
