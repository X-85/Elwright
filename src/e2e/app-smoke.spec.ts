import { expect, test } from '@playwright/test'

test.beforeEach(async ({ page }) => {
  // Only the test browser context is cleared. No real file or user directory is touched.
  await page.addInitScript(() => window.localStorage.clear())
  await page.goto('/')
  await expect(page.getByRole('button', { name: '能力工具箱' })).toBeVisible()
})

test('能力详情可点击调用，并展示预览模式的离线 SOP', async ({ page }) => {
  // weekly-report 现为进阶档位（ADR-001）：beforeEach 的 clear 是先注册的 initScript，
  // 这里追加后置种子（注册序在后），每次导航清空后重新预置累计使用 3 次
  await page.addInitScript(() =>
    localStorage.setItem('elwright-capability-uses', JSON.stringify({ 'text-stats': 3 })),
  )
  await page.reload()
  await page.getByText('周报生成', { exact: true }).click()
  const invoke = page.getByRole('button', { name: /调用/ })
  await expect(invoke).toBeEnabled()

  await invoke.click()
  // 降级提示与侧栏「预览模式 · 浏览器」徽标都含「预览模式」，必须锁定详情区内的提示
  await expect(page.locator('.detail').getByText('预览模式', { exact: false })).toBeVisible()
  await expect(page.getByText('周报生成 · 离线 SOP', { exact: true })).toBeVisible()
})

test('收藏夹与课题流程只使用隔离浏览器数据', async ({ page }) => {
  await page.getByRole('button', { name: '资源与课题' }).click()
  await expect(page.getByRole('heading', { name: '资源与课题' })).toBeVisible()

  const createFolder = page.getByTitle('新建文件夹')
  await expect(createFolder).toBeDisabled()
  await page.getByPlaceholder('新建文件夹').fill('自动化资料')
  await expect(createFolder).toBeEnabled()
  await createFolder.click()
  await expect(page.getByText('自动化资料', { exact: true })).toBeVisible()

  await page.getByPlaceholder('名称').fill('自动化资料.md')
  await page.getByPlaceholder(/本地文件路径/).fill('virtual://elwright-e2e/自动化资料.md')
  await page.getByRole('button', { name: '收藏文件', exact: true }).click()
  await expect(page.getByText('自动化资料.md', { exact: true })).toBeVisible()

  await page.locator('.workspace-tabs').getByRole('button', { name: '课题', exact: true }).click()
  await page.getByPlaceholder('新课题名称').fill('自动化测试课题')
  await page.getByPlaceholder('你想弄清什么？').fill('如何验证按钮和关键工作流？')
  await page.getByRole('button', { name: '创建课题', exact: true }).click()
  await expect(page.locator('.topic-title-input')).toHaveValue('自动化测试课题')

  await page.getByRole('checkbox', { name: /自动化资料\.md/ }).check()
  await page.getByRole('button', { name: '生成报告', exact: true }).click()
  await expect(page.getByText('这是预览模式下的离线报告草稿。', { exact: true })).toBeVisible()

  const hasHorizontalOverflow = await page.locator('.workspace-view').evaluate((view) => view.scrollWidth > view.clientWidth)
  expect(hasHorizontalOverflow).toBeFalsy()
})

test('设置按钮可打开并关闭设置中心', async ({ page }) => {
  await page.getByRole('button', { name: '打开设置' }).click()
  const settings = page.getByRole('dialog', { name: '设置' })
  await expect(settings).toBeVisible()
  await settings.getByRole('button', { name: '关闭设置' }).click()
  await expect(settings).toBeHidden()
})

test('代码浏览器：浏览器预览拒绝访问本机目录（只读降级守卫）', async ({ page }) => {
  await page.getByRole('button', { name: '代码浏览器' }).click()
  await expect(page.getByRole('button', { name: '选择项目目录' })).toBeVisible()
  await page.getByRole('button', { name: '选择项目目录' }).click()
  await expect(page.getByText('【预览模式】浏览器无法访问本机目录', { exact: false })).toBeVisible()
})
