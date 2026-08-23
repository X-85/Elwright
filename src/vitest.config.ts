import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    // theme.ts 触碰 document/localStorage/matchMedia，需要 DOM 环境
    environment: 'jsdom',
    include: ['lib/**/*.test.ts'],
  },
})
