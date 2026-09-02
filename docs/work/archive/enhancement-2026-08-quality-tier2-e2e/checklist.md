# Checklist · 工程质量第二档

标记约定（本任务首次落地）：`【自动化】`= CI/本地命令可复跑（附命令）；`【手测】`= 需人在真机执行（附路径）。

## Step 2 命令层下沉（commit 341da87）

- [x] 【自动化】`cargo build` 编译通过（命令全部迁至 core/commands.rs，main.rs 只剩 Builder/setup）— `cd src-tauri && cargo build`
- [x] 【自动化】现有 37 测试全绿（行为零变化闸门）— `cargo test`
- [x] 【自动化】clippy/fmt — `cargo clippy --all-targets -- -D warnings && cargo fmt --check`

## Step 3 IPC 冒烟（tests/terminal_ipc.rs，commit 2bcd929）

- [x] 【自动化】open 返回 id 且 channel 收到 PTY 输出（Bug#1 回归锁）
- [x] 【自动化】双会话独立：id 不同、写入互不串扰（Bug#2 类回归锁）
- [x] 【自动化】write 未知 id 返回错误
- [x] 【自动化】close 后再 write 报错 / active_count 归零
- [x] 【自动化】list_capabilities 返回 3 条内置能力
- [x] 【自动化】三平台：`cargo test --test terminal_ipc`（Windows+CI 自动走 MockBackend 分支）

## Step 4 Playwright 浏览器 e2e（src/e2e/，commit fef84da）

- [x] 【自动化】工具箱加载 3 能力 + 打开 text-stats 详情（browser bridge ↔ /api 接缝）— `cd src && npm run test:e2e`
- [x] 【自动化】降级守卫：浏览器下终端按钮不渲染 + AI 对话页显示「预览模式」提示
- [ ] 【自动化】CI frontend job 新增 e2e 步骤绿（本地等价命令已绿；ci.yml 仅 main/PR 触发，合并 main 后首次 run 回填链接）

## Step 5 文档

- [x] 【自动化】AI_CODE_AGENT_MAINTENANCE.md 标记约定段落 + engineering-quality feature 四件套 + ADR-001 + ROADMAP 第二档标完成

## 手测项（真机）

- [ ] 【手测】重构后桌面壳正常：`cd src-tauri && PATH="$HOME/.cargo/bin:$PATH" ../src/node_modules/.bin/tauri dev` → 顶栏终端按钮开 tab 可输入、AI 对话可发消息、能力详情可看（用户执行）
