import { expect, test } from '@playwright/test'
import { readFileSync } from 'node:fs'

const HELPER_SRC = readFileSync(new URL('./copy-path-invoke-helper.ts', import.meta.url), 'utf8').replace(
  /(^|\n)export\s+/g,
  '$1',
)

// B6 回归：代码浏览器文件工具条「复制文件路径」按钮。
//
// 手法：addInitScript 注入 __TAURI_INTERNALS__ mock（invoke / transformCallback /
// convertFileSrc），让前端走真实 tauriBridge 全链路；所有 code_browser_* 命令与
// plugin:dialog|open 由本文件 mock，不触真实文件系统与原生对话框。
//
// 验证：①选项目 → 展开目录 → 打开文件后工具条出现「复制路径」按钮；
// ②点击后剪贴板拿到 <项目根><sep><相对路径>（Windows 反斜杠根目录用 \
// 拼接，joinProjectPath 行为）；③修复前按钮是纯图标（13px 图标、无文字），
// 用户点验时找不到——现补文字标签，用例按 getByRole name '复制路径' 守护。

const PROJECT_ROOT = ['D:', 'demo', 'myproj'].join(String.fromCharCode(92))
const FILE_REL = 'src/main/java/com/example/App.java'

test.beforeEach(async ({ page }) => {
  await page.addInitScript((body: string) => {
    const makeInvoke = new Function(body + ';\nreturn makeInvoke;')() as () => (
      cmd: string,
      args?: Record<string, unknown>,
    ) => unknown
    ;(window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__ = {
      invoke: (cmd: string, args?: Record<string, unknown>) =>
        Promise.resolve(makeInvoke()(cmd, args ?? {})),
      transformCallback: (cb: number) => cb,
      convertFileSrc: (p: string) => p,
      // App.vue 在 bridge.kind==='tauri' 时调 getCurrentWindow()，需要 metadata
      metadata: { currentWindow: { label: 'main' }, currentWebview: { label: 'main' } },
    }
  }, HELPER_SRC)
})

test('复制路径按钮可见可点，剪贴板拿到项目根拼接的绝对路径', async ({ page, context }) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write'])
  await page.goto('/')
  await page.getByRole('button', { name: '代码浏览器' }).click()

  // 选择项目（dialog mock 返回 D:\demo\myproj）
  await page.getByRole('button', { name: /选择项目/ }).click()
  await expect(page.getByText('myproj').first()).toBeVisible()

  // 展开目录 src（懒加载一层树）
  await page.getByRole('button', { name: /src/ }).first().click()
  // 打开文件
  await page.getByRole('button', { name: /App\.java/ }).first().click()
  // 文件工具条出现
  const copyBtn = page.getByRole('button', { name: '复制路径' })
  await expect(copyBtn).toBeVisible()

  // 点击并读剪贴板
  await copyBtn.click()
  const clip = await page.evaluate(() => navigator.clipboard.readText())
  expect(clip).toBe(PROJECT_ROOT + '\\' + FILE_REL)

  // 通知反馈
  await expect(page.getByText('已复制', { exact: true })).toBeVisible()

  // 留证截图（交付到任务目录）
  await page.screenshot({
    path: '../docs/work/active/bugfix-2026-09-code-browser-copy-path/screenshots/copy-path-button.png',
    fullPage: true,
  })
})
