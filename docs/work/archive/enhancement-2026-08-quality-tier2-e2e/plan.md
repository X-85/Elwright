# Enhancement · 工程质量治理第二档 · 分层 e2e 冒烟

基线：main `af1577b`（v0.1.5 已发版，CI 7/7 全绿）。
分支：`enhancement/2026-08-quality-tier2-e2e`

## 目标（ROADMAP 第二档原文）

1. 「tauri-driver 或 Playwright e2e 冒烟（打开终端 → 敲命令 → 断言输出，抓集成断线类 bug）」
2. 「验证清单加『自动化覆盖/需手测』标记，防止未执行的验证被标通过」

## 选型结论（详见 docs/features/engineering-quality/decisions/ADR-001）

放弃 tauri-driver（Windows CI 无 WinAppDriver 且需管理员、Linux 有 xvfb/焦点前科），分两层：

- **Layer 1 · IPC 冒烟**（`src-tauri/tests/terminal_ipc.rs`）：tauri mock runtime + 真实 IPC 协议（`get_ipc_response`），channel 参数走真 `CommandArg` 解析（Bug #1 出错路径）。macOS/Linux 用真 LocalBackend——真 PTY 跑通「开终端 → 敲命令 → 断言输出」；Windows+CI 用 MockBackend（ConPTY 惯例）。随 rust matrix 三平台跑。
- **Layer 2 · Playwright 浏览器 e2e**（`src/e2e/`）：黑盒验证 browser bridge ↔ vite /api 接缝与降级守卫（终端按钮不渲染、AI 对话预览模式提示）。CI frontend job 跑。
- GUI 桌面壳（xterm 渲染、tab 交互）保持【手测】标记——两层覆盖接缝逻辑，像素/交互留给真机。

## 前置重构（Step 2，行为零变化）

20 个 `#[tauri::command]` 从 main.rs（bin crate，tests/ 无法触达）下沉到 `core/commands.rs`，泛型 `<R: Runtime>`；statics 改 `AppCtx` + `.manage()` 注入。闸门：编译 + 现有 37 测试全绿。

## 步骤

1. 任务目录（本文件）+ checklist/verification
2. 命令层下沉重构 + spike（generate_handler 跨 crate、mock 下 async 命令）
3. IPC 冒烟测试（5 用例，Bug#1/#2 回归锁）
4. Playwright e2e（2 场景）+ CI frontend job 接入
5. 标记惯例写入 AI_CODE_AGENT_MAINTENANCE.md + engineering-quality feature 文档 + ADR + ROADMAP
6. 五道闸全绿 → 分主题提交 → 推分支
