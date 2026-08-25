# Checklist · 工作工具栏第一阶段

标记约定：`【自动化】`= CI/本地命令可复跑（附命令）；`【手测】`= 需人在真机执行（附路径）。

## Step 2 Rust core（core/workbench.rs）

- 【自动化】Todo 往返：add → list → toggle → remove，id 单调递增，completedAt 正确维护 — `cd src-tauri && cargo test workbench`
- 【自动化】Note 往返：save → get → list_dates 倒序；非法日期（`../evil`、`2026-8-5`、空串）中文报错 — 同上
- 【自动化】文件落点：`<ELWRIGHT_USER_ROOT>/todos.json` 与 `<ELWRIGHT_USER_ROOT>/notes/YYYY-MM-DD.md` — 同上

## Step 3 IPC 命令（core/commands.rs + main.rs）

- 【自动化】7 条命令经 mock runtime 真协议冒烟（todo 增查勾删往返、note 存取往返、非法日期报错、未知 id 报错）— `cd src-tauri && cargo test --test workbench_ipc`
- 【自动化】全量回归 — `cargo test`

## Step 4 前端（bridge.ts + WorkbenchView.vue + App.vue）

- 【自动化】browserBridge 模拟存储（刷新即失口径）— `cd src && npm test`
- 【自动化】构建无类型错误 — `npm run build`

## Step 5 e2e（src/e2e/）

- 【自动化】工作台视图：加 Todo → 出现 → 勾选划线 → 删除消失；今日记录输入 → 自动保存提示；预览模式徽标 — `cd src && npm run test:e2e`

## Step 6 文档与收尾

- 【自动化】docs/features/workbench/ 四件套更新（README 当前状态、behavior、architecture、changelog）+ ROADMAP 进行中登记与范围调整记录

## 手测项（真机）

- 【手测】桌面壳持久化：`cd src-tauri && PATH="$HOME/.cargo/bin:$PATH" ../src/node_modules/.bin/tauri dev` → 工作台加几条 Todo + 写今日记录 → 完全退出重启 app → 数据仍在；`ls ~/.elwright/` 看到 `todos.json` 与 `notes/2026-08-25.md`（用户执行）
