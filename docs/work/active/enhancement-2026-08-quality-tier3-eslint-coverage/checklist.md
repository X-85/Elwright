# 工程质量第三档 ESLint + Coverage — Checklist

> 与 plan.md 一一对应；执行顺序自上而下。

## 阶段一：ADR 落地（本 PR）

- [x] ADR-002-eslint-and-coverage.md 写完，决策=ESLint 9 flat + v8 coverage 70/60 阈值
- [x] task dir 三件套建好（plan/checklist/verification）
- [ ] 本地复阅 ADR；commit 到 feat/engineering-tier3-eslint-coverage-adr
- [ ] 开 PR（ADR only）；CI 7/7 不受影响（无代码改动）
- [ ] 合并 main；删除分支

## 阶段二：实施 PR

> 独立分支 `feat/engineering-tier3-eslint-coverage-impl`；合并 ADR 后再开。

### 配置层

- [ ] `npm install -D eslint typescript-eslint eslint-plugin-vue @vitest/coverage-v8`
- [ ] `src/eslint.config.js`：flat config，规则三组（no-unused-vars + vue/no-unused-components + no-explicit-any warn）
- [ ] `src/vitest.config.ts` 加 coverage 块：provider v8，include `lib/**/*.ts`，thresholds 70/70/70/60
- [ ] `src/package.json` 加 scripts：`lint`、`lint:fix`、`test:coverage`

### 清零现有告警

- [ ] 跑 `npm run lint --fix` 残量看一遍
- [ ] 未使用 import 删净
- [ ] 未使用 `<script setup>` 变量 / `defineProps` 字段删净
- [ ] 任何 `any` 加 narrow 类型或 `unknown` + 类型守卫；警告不阻断但要明确
- [ ] `npm run lint` 退出码 0

### Coverage 基线确认

- [ ] `npm run test:coverage` 第一次跑，记录实际百分比
- [ ] 若实际 < 阈值：调整阈值至「现有存量能过 + 后续回归能抓」的折中值；plan 文档同步更新
- [ ] 若实际 ≥ 阈值：直接定稿 70/60
- [ ] `npm run test:coverage` 退出码 0

### CI 接缝

- [ ] `.github/workflows/ci.yml` frontend job 加 `npm run lint` 与 `npm run test:coverage` 两步
- [ ] 矩阵三平台（macOS / Ubuntu / Windows）一致
- [ ] 不新增 job；不引入 allow-failure

### 文档

- [ ] `docs/features/engineering-quality/README.md` 表格第三档状态 → 已完成；加 ADR-002 链接
- [ ] `docs/features/engineering-quality/changelog.md` 增第三档条目（日期 2026-08-31）
- [ ] `docs/ROADMAP.md` 第三档条目 ~~删除线~~ + ADR-002 链接；进行中 / 未发版 / 历史 段同步 Q20

### 验收

- [ ] 本地 6 道闸（lint + test:coverage + vitest + vite build + （playwright 由 CI 跑） + ...）全绿
- [ ] `git push origin feat/engineering-tier3-eslint-coverage-impl`
- [ ] 开 PR；CI 7/7 + frontend 新两步全绿
- [ ] squash merge + --delete-branch
- [ ] main 拉取，git status 干净

## 阶段三：台账

- [ ] `session/index.md` 加 Q20 条目（已完成）
- [ ] `session/events.md` 加 Q20 第1次处理（ADR）+ 第2次处理（实施）
- [ ] commit + push 至 main
