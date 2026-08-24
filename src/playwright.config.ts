import { defineConfig } from '@playwright/test'

// 浏览器级 e2e：只测浏览器预览壳（vite dev 的 /api/* 只读端点），
// 不测 Tauri 桌面壳——那层由 src-tauri/tests/terminal_ipc.rs 的
// mock-runtime IPC 测试覆盖（分层见 docs/features/engineering-quality）。
export default defineConfig({
  testDir: './e2e',
  timeout: 30_000,
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? 'github' : 'list',
  use: {
    baseURL: 'http://localhost:5173',
  },
  webServer: {
    command: 'npm run dev -- --port 5173 --strictPort',
    url: 'http://localhost:5173',
    reuseExistingServer: !process.env.CI,
    timeout: 60_000,
  },
  projects: [{ name: 'chromium', use: { browserName: 'chromium' } }],
})
