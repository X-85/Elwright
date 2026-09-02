import { expect, test } from '@playwright/test'
import { readFileSync } from 'node:fs'

// B4 回归：代码浏览器搜索四子项。
//
// 手法：复用 copy-path 的 __TAURI_INTERNALS__ 注入架构（helper 以源码字符串
// 嵌入，走真实 tauriBridge 全链路）；code_browser_search 由本文件 mock，
// 用可编程延迟与返回值验证四子项：
// ①输入停顿 300ms 自动搜索（不等回车/按钮）——mock 调用计数断言；
// ②结果以文件名为主、完整路径弱化为次要文本；
// ③「清除搜索」按钮一键清空输入与结果（回归子项④「清不掉」）；
// ④竞态：搜索中输入变化后旧响应不得覆盖（旧实现 Promise 卡死按钮置灰）。

const HELPER_SRC = readFileSync(new URL('./copy-path-invoke-helper.ts', import.meta.url), 'utf8').replace(
  /(^|\n)export\s+/g,
  '$1',
)

type Log = { cmd: string; rel?: string }[]
let ipcLog: Log = []
let searchDelayMs = 0
let searchResult: unknown[] = []

async function injectMock(page: import('@playwright/test').Page) {
  await page.addInitScript(
    ({ body, state }: { body: string; state: { delay: number; result: unknown[] } }) => {
      const makeInvoke = new Function(body + ';\nreturn makeInvoke;')() as () => (
        cmd: string,
        args?: Record<string, unknown>,
      ) => unknown
      const base = makeInvoke()
      ;(window as unknown as { __ELWRIGHT_IPC_LOG__: { cmd: string; rel?: string }[] }).__ELWRIGHT_IPC_LOG__ = []
      ;(window as unknown as { __ELWRIGHT_SEARCH_STATE__: { delay: number; result: unknown[] } }).__ELWRIGHT_SEARCH_STATE__ =
        state
      ;(window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__ = {
        invoke: (cmd: string, args?: Record<string, unknown>) => {
          const log = (window as unknown as { __ELWRIGHT_IPC_LOG__: { cmd: string; rel?: string }[] })
            .__ELWRIGHT_IPC_LOG__
          log.push({ cmd, rel: args?.rel as string | undefined })
          if (cmd === 'code_browser_search') {
            const st = (window as unknown as { __ELWRIGHT_SEARCH_STATE__: { delay: number; result: unknown[] } })
              .__ELWRIGHT_SEARCH_STATE__
            if (st.delay > 0)
              return new Promise((resolve) =>
                setTimeout(() => resolve(st.result), st.delay),
              )
            return Promise.resolve(st.result)
          }
          return Promise.resolve(base(cmd, args ?? {}))
        },
        transformCallback: (cb: number) => cb,
        convertFileSrc: (p: string) => p,
        metadata: { currentWindow: { label: 'main' }, currentWebview: { label: 'main' } },
      }
    },
    { body: HELPER_SRC, state: { delay: searchDelayMs, result: searchResult } },
  )
}

test.beforeEach(async ({ page }) => {
  ipcLog = []
  searchDelayMs = 0
  searchResult = []
  await injectMock(page)
  await page.goto('/')
  await page.getByRole('button', { name: '代码浏览器' }).click()
  await page.getByRole('button', { name: /选择项目/ }).click()
  await expect(page.locator('.cb-project')).toHaveText('myproj')
})

// addInitScript 的参数按值序列化：beforeEach 里改闭包变量不生效。
// 统一在页内通过 __ELWRIGHT_SEARCH_STATE__（对象引用）设定用例数据。
async function setSearch(page: import('@playwright/test').Page, result: unknown[], delay = 0) {
  await page.evaluate(
    ({ result, delay }) => {
      const st = (window as unknown as { __ELWRIGHT_SEARCH_STATE__: { delay: number; result: unknown[] } })
        .__ELWRIGHT_SEARCH_STATE__
      st.delay = delay
      st.result = result
    },
    { result, delay },
  )
}

test('即时模糊搜索：输入停顿自动触发，无需回车或按钮', async ({ page }) => {
  await setSearch(page, [{ path: 'src/App.java', name: 'App.java', line: 0, snippet: '' }])
  const input = page.locator('.cb-search-input').first()
  await input.fill('app')
  // 300ms 防抖 + IPC 往返
  await expect(page.locator('.cb-hits .cb-hit').first()).toBeVisible({ timeout: 5000 })
  const log = (await page.evaluate(() => window.__ELWRIGHT_IPC_LOG__)) as { cmd: string }[]
  expect(log.filter((e) => e.cmd === 'code_browser_search').length).toBeGreaterThanOrEqual(1)
})

test('结果以文件名为主，完整路径弱化展示', async ({ page }) => {
  await setSearch(page, [
    { path: 'src/main/java/com/example/App.java', name: 'App.java', line: 0, snippet: '' },
  ])
  await page.locator('.cb-search-input').first().fill('app')
  const hit = page.locator('.cb-hits .cb-hit').first()
  await expect(hit).toBeVisible({ timeout: 5000 })
  // 主文本是文件名
  await expect(hit).toContainText('App.java')
  // 完整路径在次要 span 中
  const pathSpan = hit.locator('.cb-hit-path')
  await expect(pathSpan).toHaveText('src/main/java/com/example/App.java')
  // 留证截图
  await page.screenshot({
    path: '../docs/work/active/bugfix-2026-09-code-browser-search-ux/screenshots/search-result-filename-first.png',
  })
})

test('清除搜索：一键清空输入与结果，恢复按钮可用', async ({ page }) => {
  await setSearch(page, [{ path: 'src/App.java', name: 'App.java', line: 0, snippet: '' }])
  const input = page.locator('.cb-search-input').first()
  await input.fill('app')
  await expect(page.locator('.cb-hits .cb-hit').first()).toBeVisible({ timeout: 5000 })

  const clearBtn = page.getByRole('button', { name: '清除搜索' })
  await expect(clearBtn).toBeVisible()
  await clearBtn.click()

  await expect(input).toHaveValue('')
  await expect(page.locator('.cb-hits')).toHaveCount(0)
  // 子项④回归核心：无卡死的进行中状态——再次输入可正常触发新一轮搜索
  await setSearch(page, [{ path: 'src/next.java', name: 'next.java', line: 0, snippet: '' }])
  await input.fill('next')
  await expect(page.locator('.cb-hits .cb-hit').first()).toBeVisible({ timeout: 5000 })
  await expect(page.locator('.cb-hits .cb-hit').first()).toContainText('next.java')
})

test('竞态防护：慢响应不阻塞后续输入，清除后不再覆盖', async ({ page }) => {
  await setSearch(page, [{ path: 'src/slow-hit.java', name: 'slow-hit.java', line: 0, snippet: '' }], 3000)
  const input = page.locator('.cb-search-input').first()
  await input.fill('slow')
  // 立即清空（响应还在路上）
  await page.getByRole('button', { name: '清除搜索' }).click()
  await expect(input).toHaveValue('')
  // 3s 后慢响应返回，不得把结果塞回来（seq 已失效）
  await page.waitForTimeout(3500)
  await expect(page.locator('.cb-hits')).toHaveCount(0)
})
