# 集成终端（Integrated Terminal）

> 状态：**进行中**（v1 仅本地 shell；SSH/远端会话留待 v1.x，由 `TerminalBackend` trait 扩展）。

Elwright 桌面壳内嵌一个集成终端，类比 VS Code / Cursor / Tabby / Codex Desktop 的底部抽屉终端。
桌面应用里可以直接手动调用脚本、打开 Claude Code（或其他任意交互式程序），并与现有 script 能力联动（"在终端中运行"）。

## 阅读顺序

1. [behavior.md](./behavior.md) —— 当前业务规则、用户能做什么、限制是什么
2. [architecture.md](./architecture.md) —— Rust 后端 trait、前端组件、Tauri Channel 流式通道、SSH 扩展点
3. [changelog.md](./changelog.md) —— 已上线变化
4. [decisions/ADR-001-terminal-stack.md](./decisions/ADR-001-terminal-stack.md) —— 为什么是 xterm.js + portable-pty

## 代码位置

- Rust：`src-tauri/src/core/terminal/{mod.rs, backend.rs, local.rs, ipc.rs}`
- Tauri IPC：`src-tauri/src/main.rs`（新增 `open_terminal_session` / `terminal_write` / `terminal_resize` / `terminal_close` 等命令）
- 前端组件：`src/components/TerminalPanel.vue` + `src/components/TerminalView.vue`
- 桥接：`src/lib/bridge.ts`（新增 `TerminalSession` / `openTerminal` / `terminalWrite` / `terminalResize` / `terminalClose`）

## 相关任务

- [feature-2026-08-integrated-terminal](../../work/active/feature-2026-08-integrated-terminal/)（计划、checklist、验证）