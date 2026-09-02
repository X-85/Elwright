import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    // theme.ts 触碰 document/localStorage/matchMedia，需要 DOM 环境
    environment: 'jsdom',
    include: ['lib/**/*.test.ts'],
    coverage: {
      provider: 'v8',
      include: ['lib/**/*.ts'],
      exclude: [
        'lib/**/*.test.ts',
        'lib/__tests__/**',
        // bridge.ts 是 IPC/浏览器双实现 facade，由 IPC mock runtime + Playwright e2e 覆盖；
        // 同构于 ADR-002 「弃选把 .vue 纳入门槛」——facade 层由更高层测试兜底。
        'lib/bridge.ts',
      ],
      reporter: ['text', 'html'],
      thresholds: {
        lines: 70,
        functions: 70,
        statements: 70,
        branches: 60,
      },
    },
  },
})
