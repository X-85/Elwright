# 合并准备：rebase 到 main 的冲突清单与 e2e 去重方案

> 2026-08-30 用 `git merge-tree` 预演生成（当时 main = `a58705d`，tier2 已并入，
> workbench 待合并）。
>
> **执行进度（2026-08-30）**：已提前对 tier2 状态的 main 完成一轮 rebase（force-with-lease
> 推送至 `cbe9a75`），全部冲突按本清单解决，五道闸全绿（cargo 42+6 / clippy / fmt /
> vitest 22 / e2e 8 场景合跑）。适配项：内置注册表 3→4 条（session-ledger）后
> terminal_ipc 与 app.spec 计数断言同步更新。**剩余**：无。
>
> **第二轮 rebase（2026-08-30，workbench 合并后）**：已完成。预演 6 冲突
> （ROADMAP/commands.rs/mod.rs/main.rs/App.vue/bridge.ts），全部按并集/取优解决——
> App.vue 以本分支 app-chrome 壳为基底并回工作台侧栏入口；main.rs/mod.rs/commands.rs
> 为 workbench todo/note 命令与本分支 workspace 命令并集。五道闸全绿
> （cargo 47+6+4 / clippy / fmt / vitest 26 / e2e 9 场景）。修复了并集脚本对
> bridge.ts 块尾截断的误拼接（1d20db8）。分支已 force-with-lease 推 origin，
> 可进入合并流程。

## 预演冲突文件（6 个）

| 文件 | 冲突类型 | 解法 |
| --- | --- | --- |
| `src/playwright.config.ts` | add/add | 取 main 版（5273 端口 + 踩坑注释）；可顺手合入本分支的 viewport 1440×980 与 trace: retain-on-failure |
| `.github/workflows/ci.yml`（自动合并但有 e2e 重复） | 语义重复 | 删本分支独立 `e2e` job（含 playwright-report artifact），保留 main 集成 Playwright step |
| `.gitignore` | content | 取并集；playwright-report/ 等本分支条目若 main 未有则补 |
| `AGENTS.md` / `docs/ROADMAP.md` | content | 以 main 为基础，重放 `94dda1c` 的文档同步（重放前核对 main 是否已覆盖） |
| `src-tauri/src/main.rs` | content | 逐块手解（本分支注册的 IPC 命令 + main 的 tier2 IPC 测试入口并存） |
| `src/App.vue` | content | 逐块手解（窗口控制/消息/工作台布局与本分支渐进能力 UI 并存） |

## e2e 去重决策（Q5 遗留问题的落地结论）

- **配置**：只留 main 的一份 `playwright.config.ts`（testDir `./e2e` glob 自动收两份 spec）。
- **场景**：两边零重叠，都保留——
  - main `app.spec.ts`（5）：工具箱加载 / 筛选搜索 / 知识详情接缝 / 终端按钮降级 / AI 对话降级；
  - 本分支 `app-smoke.spec.ts`（3）：能力调用离线 SOP / 收藏夹课题隔离数据 / 设置中心开关。
  - `app-smoke.spec.ts` 无硬编码 URL，直接兼容 5273 端口。
- **CI**：删本分支独立 `e2e` job，`npm run test:e2e` script 定义以 main 为准（冲突时手核 package.json）。

## 执行后验收

五道闸全绿：cargo test / clippy / fmt / vitest / e2e（含 8 个浏览器场景）。
