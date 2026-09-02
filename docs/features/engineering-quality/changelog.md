# changelog — engineering-quality

## 2026-08-31 — 第三档：ESLint 10 + vitest coverage v8（PR #14）

- 新增 ESLint 10 flat config（`src/eslint.config.js`）：`typescript-eslint` + `eslint-plugin-vue`；规则三组——`no-unused-vars`（`_` 前缀豁免）+ `vue/no-unused-components` + `vue/no-unused-vars` + `no-explicit-any` warn；TS/Vue 下 `no-undef` 关闭；vue 模板风格规则 off（prettier 独立 ADR）
- 新增 vitest coverage v8 provider（`src/vitest.config.ts`）：`include lib/**/*.ts`，`exclude lib/bridge.ts`（facade 由 IPC mock + e2e 兜底）+ test 文件；thresholds lines/functions/statements 70% / branches 60%
- vitest 测试 51 → 60：新增 9 例覆盖 preferences localStorage / DOM 副作用
- 清零 ESLint 警告时连带清理 6 个真未用 import/变量 + 1 个 watch 形参 `_id` 化
- TypeScript 7.0.2 → 6.0.3：typescript-eslint 8.68 显式拒绝 TS 7（启动期硬错），vite 8 / vitest 4 peer 与 TS 版本无关，安全降级
- CI frontend job 新增 `npm run lint` 与 `npm run test:coverage` 两步；三平台 matrix 一致
- 本地六道闸：eslint 0 / vitest 60 + coverage 95/76/100/95 / vite build / cargo fmt / cargo clippy / cargo test 全绿；CI 7/7 全绿
- 决策详见 [ADR-002](decisions/ADR-002-eslint-and-coverage.md)

## 2026-08-24 — 第二档：分层 e2e 冒烟（PR #9）

- IPC 层 mock runtime + Playwright 浏览器层；详见 [ADR-001](decisions/ADR-001-e2e-layering.md)

## 2026-08-23 — 第一档：CI clippy + rustfmt

- GitHub Actions `lint` job：cargo fmt --check + clippy --all-targets -D warnings
