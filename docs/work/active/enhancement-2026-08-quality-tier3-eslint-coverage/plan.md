# 工程质量第三档 ESLint + Coverage — Plan

> 对应 ADR-002-eslint-and-coverage.md。背景与决策以 ADR 为准，本文件只列实施清单与依赖。

## 范围

最小必要引入 ESLint 9 flat config + @vitest/coverage-v8，CI 加 `lint` 与 `test:coverage` 两步。不引 prettier；不把 .vue 纳入覆盖率门槛。

## 交付物清单

### 1. 配置文件（src/）

- [ ] `src/eslint.config.js`（flat config）：tseslint + vue plugin，规则三组（见 ADR）
- [ ] `src/vitest.config.ts` 加 `test.coverage` 块（provider v8 + thresholds：lines/functions/statements 70%，branches 60%；include `lib/**/*.ts`）
- [ ] `src/package.json` 加 devDependencies：`eslint`、`typescript-eslint`、`eslint-plugin-vue`、`@vitest/coverage-v8`
- [ ] `src/package.json` 加 scripts：`lint` (`eslint .`)、`lint:fix` (`eslint . --fix`)、`test:coverage` (`vitest run --coverage`)
- [ ] `src/.gitignore` 不需要改（不产生新忽略）；`dist/` 已忽略

### 2. 现有代码清零 lint 告警

- [ ] 实施 PR 一次跑 `npm run lint:fix` + 手修残余，使 lint 零告警
- [ ] 若发现未使用 import/变量，**就地删除**而非 `// eslint-disable`
- [ ] 任何 `.vue` 模板未使用 ref 用 `vue/no-unused-vars` 兜底

### 3. CI 接缝（.github/workflows/ci.yml）

- [ ] 现有 frontend job 在 `npm ci && npm run build` 之前加 `npm run lint`
- [ ] 在 `npm run test:e2e` 之前加 `npm run test:coverage`
- [ ] 矩阵三平台（macOS / Ubuntu / Windows）一致执行
- [ ] 任一非绿 fail；不设 allow-failure

### 4. 文档

- [ ] `docs/features/engineering-quality/README.md`：第三档状态从「按需」改为「已完成」并加 ADR-002 链接
- [ ] `docs/features/engineering-quality/changelog.md`：第三档条目
- [ ] `docs/ROADMAP.md`：
  - 第三档条目加 ~~删除线~~ + 链接到 ADR-002
  - 「进行中」段补 Q20
  - 「未发版」段补 Q20
  - 「历史」段补 Q20

## 验收阈值

- 本地 `npm run lint` 退出码 0，零告警
- 本地 `npm run test:coverage` 退出码 0，覆盖率 ≥ 阈值
- CI frontend job 6/6（lint + test:coverage + vitest + test:e2e + build + Playwright smoke）全绿
- CI 三平台通过；macOS dmg + Windows msi 流水线不阻塞

## 不做（明确范围外）

- 不引 prettier
- 不把 .vue / .css 纳入 coverage 门槛
- 不改 Rust 侧 CI（已稳：clippy + fmt + cargo test/build）
- 不引入 husky / lint-staged（CI 已能挡，留作团队规模扩大后的独立 ADR）
- 不重构 components/（顺带发现的可读性问题记录到 PENDING-REAL-MACHINE-CHECKLIST.md「工程债」段，本次不处理）
