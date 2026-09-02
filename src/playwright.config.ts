import { defineConfig } from '@playwright/test'

// 浏览器级 e2e：只测浏览器预览壳（vite dev 的 /api/* 只读端点），
// 不测 Tauri 桌面壳——那层由 src-tauri/tests/terminal_ipc.rs 的
// mock-runtime IPC 测试覆盖（分层见 docs/features/engineering-quality）。
// 端口用 5273（非 vite 默认）：5173 常被本机其他工作区的 dev server 占用，
// reuseExistingServer 会把测试打到别人的 UI 上（2026-08-24 实际踩过）。
export default defineConfig({
  testDir: './e2e',
  timeout: 30_000,
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? 'github' : 'list',
  use: {
    baseURL: 'http://localhost:5273',
    // 本机（公司网关封 Chrome for Testing CDN）下载不了 Playwright 自带浏览器，
    // 改用系统 Chrome 通道；CI 与家里 mac 无此限制，channel 缺省时行为不变。
    ...(process.env.PLAYWRIGHT_USE_SYSTEM_CHROME ? { channel: 'chrome' as const } : {}),
  },
  webServer: {
    command: 'npm run dev -- --port 5273 --strictPort',
    url: 'http://localhost:5273',
    reuseExistingServer: !process.env.CI,
    timeout: 60_000,
  },
  projects: [{ name: 'chromium', use: { browserName: 'chromium' } }],
})
