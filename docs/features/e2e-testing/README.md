# 浏览器端到端测试

Playwright 浏览器冒烟测试验证 Elwright 预览模式中真实可点击的页面工作流。它补充 Rust 单元测试和 Vitest 纯逻辑测试，不替代 Tauri 原生集成测试。

入口：`src/playwright.config.ts`、`src/e2e/`、`npm run test:e2e` 和 CI 的 `Browser smoke (Playwright)` job。首次本机执行前运行 `npx playwright install chromium`。当前规则见 [behavior.md](./behavior.md)，隔离与系统边界见 [architecture.md](./architecture.md)。
