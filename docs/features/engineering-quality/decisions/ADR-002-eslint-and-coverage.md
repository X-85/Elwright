# ADR-002 — 第三档：ESLint 启用 + 覆盖率门槛（vitest v8）

- 状态：已采纳（2026-08-31）
- 关联任务：`docs/work/active/enhancement-2026-08-quality-tier3-eslint-coverage/`
- 关联前置：ADR-001 e2e 分层（IPC mock + Playwright）

## 背景

工程质量第三档在 ROADMAP 中描述为「eslint（风格已统一，低优先）；覆盖率门槛（等测试有存量）」。到 2026-08-31，触发条件满足：

- 前端 vitest 单测存量 51 例（10 个 test file），覆盖 `lib/` 下的纯逻辑模块（safeMarkdown / theme / bridge / preferences / patch / chatProposal / codeLinks / codeHighlight / profileName / workbench-bridge）。**测试有存量**，覆盖率门槛可以立竿见影抓到回归。
- 风格统一靠的是 vite + tsc，无显式 lint。前两轮改动（PR #11/#12）已多次出现未使用 import 与 console 残留——靠 reviewer 肉眼看，不可持续。
- 第三档原话「低优先」指**立项时机**不急，不指**不立**。工程质量本就按档滚动，第三档达到前置条件就开。

候选路线两条：

1. **完整 lint 工具链（eslint + prettier + 风格统一自动化）**
2. **ESLint（不引 prettier）+ vitest v8 coverage 阈值**

## 决策

**采用路线 2：最小必要引入。**

- **ESLint 9 flat config** + `typescript-eslint` + `eslint-plugin-vue`；启三类规则：
  - `@typescript-eslint/no-unused-vars`（含 `_` 前缀豁免）——直接抓 PR #11/#12 反复出现的未使用 import。
  - `vue/no-unused-components` / `vue/no-unused-vars`——Vue SFC 模板/脚本未引用。
  - `@typescript-eslint/no-explicit-any` warn——any 散落要可见，但不阻断构建。
- **不在本档引 prettier**：当前 `style.css` 与 `.vue` `<style>` 块的手写风格不强制统一；`tsc` + vitest 已能挡住结构性错误；多一个格式化工具就多一处 CI 抖动源。**留作独立 ADR**（如果未来团队规模到需要的话）。
- **vitest coverage 用 `@vitest/coverage-v8`**：v8 原生 Node 8 起的内置 coverage，无需 istanbul 字节码插桩，启动快、产物体积小。
- **覆盖率门槛**：仅对 `src/lib/**/*.ts`（被单测覆盖的纯逻辑）设定 `lines: 70% / functions: 70% / statements: 70% / branches: 60%`。**`src/components/**/*.vue` 与 `src/App.vue` / `src/main.ts` 不计入门槛**——这两层属于浏览器/组件层，由第二档 Playwright e2e 与 vitest 行为冒烟覆盖（详见下方「为何不把 .vue 纳入门槛」）。
- **CI 接缝**：在现有 frontend job 加两步——`npm run lint` + `npm run test:coverage`；任一非绿即 fail。**不新增 CI job**，沿用 matrix。

## 弃选与理由

### 弃 prettier + eslint-config-prettier

- 现存风格没有强约束文档，加 prettier 会把整个仓库一次性大改（数百个 .vue/.ts 行尾/缩进/引号变更）——这本身是个独立 PR，不应混进 lint 启用。
- vite/tsc 已是事实上的结构门槛，prettier 只补空白——ROI 低。

### 弃 istanbul / c8

- `@vitest/coverage-istanbul` 需要 babel/swc 插桩，启动与产物都重于 v8。
- `c8`（npm 上独立 c8）功能重叠 v8 provider，且 vitest 自身已集成 v8 provider，统一在 vitest 进程内跑更顺。

### 弃 SonarQube / 代码异味扫描器

- 重型工具，单人维护项目不值得引入。eslint + coverage + 第二档 e2e 已覆盖 ROI 高的部分。

### 弃把 .vue 文件纳入 coverage 门槛

- 覆盖率门槛的真正作用是「发现单测覆盖盲点」——盲点几乎都在纯逻辑 lib 里。`components/*.vue` 由：
  - Playwright e2e 跑真 UI（`src/e2e/app-smoke.spec.ts` 端到端冒烟）覆盖集成层；
  - vitest 测试 `bridge.ts` 覆盖 IPC 接缝（bridge.test.ts + workbench-bridge.test.ts 共 11 例）；
  - vue 模板渲染覆盖率本身意义不大（template 表达式多走 props 计算）。
- 把 .vue 拉进门槛会引入「为了覆盖率写无效测试」的逆向激励，违反工程治理的最小必要原则。

## 后果

- **CI 闸门加 2 项**：`npm run lint` 与 `npm run test:coverage`；现有 4 项闸门（vitest + playwright + vite build + cargo test/build/clippy/fmt）变 6 项。
- **覆盖率基线**：首次跑 `npm run test:coverage` 会输出实际百分比——以实施 PR 实测值为门槛基准，不预设高目标（70% 是「以现有存量能过、后续回归能抓」的折中值）。
- **lint 首次启用会有遗留告警**：实施 PR 一并清零；禁止「先合并 lint 配置、留 warning 后修」的二阶段切法—— lint 启用那一刻起就是 fail 级。
- **第三方依赖类型**：eslint 工具链不进入运行时产物（devDependencies），桌面壳体积不受影响。
- **真机点验**：lint 与 coverage 都在 CI 跑，本地 `npm run lint` 与 `npm run test:coverage` 双绿即可验证；不依赖桌面壳。

## 影响面

- `src/package.json`：新增 devDependencies（eslint、typescript-eslint、eslint-plugin-vue、@vitest/coverage-v8）；新增 scripts（`lint`、`lint:fix`、`test:coverage`）。
- `src/eslint.config.js`（flat config）：规则集如上。
- `src/vitest.config.ts`：新增 `test.coverage` 配置（provider: 'v8' + thresholds）。
- `.github/workflows/ci.yml`：frontend job 加 `npm run lint` 与 `npm run test:coverage` 两步。
- 不改动 `src-tauri/` 与 `resources/`——本档纯前端工程治理。
