# ADR-001: 终端技术栈选型

**日期**：2026-08-22
**状态**：已采纳（v1 实施中）
**决策者**：Elwright 维护方

## 背景

Elwright 桌面壳需要集成终端能力，让用户在应用内手动调用脚本、运行 Claude Code 等 TUI 程序。同时为后续 SSH/远端会话（xshell-like）预留扩展空间。

需求：
- 体验好（类比 VS Code / Cursor）
- 不闪屏（终端连续输出时无视觉跳变）
- 内存可控
- 未来可承载 SSH 后端

## 候选方案

### A. xterm.js + portable-pty + Tauri Channel

- 前端：xterm.js（WebGL 渲染器默认）+ addon-fit
- 后端：`portable-pty`（Rust 官方主流，wezterm 团队维护）
- IPC：Tauri `Channel<&[u8]>` 二进制流
- **优点**：VS Code / Cursor / Codex Desktop 同款体验；Tauri 生态验证（kerminal / hermes-ide 同样栈）；portable-pty 跨平台稳定（含 ConPTY）；SSH 扩展只需新增 `SshBackend: TerminalBackend`
- **缺点**：WebGL 在老旧显卡 / 远程桌面偶发上下文丢失（xterm.js 已自动 fallback DOM）

### B. libghostty 嵌入

- 前端：libghostty-vt 渲染（或 Ghostty 自身 GUI）
- 后端：libghostty 处理 PTY/SSH/协议层
- **优点**：理论体验最好（Ghostty 原生品质）
- **缺点**：官方未打 tag，API 在变动；Zig C-ABI 集成复杂；生产嵌入风险高

### C. 仅开外部终端（GitButler 路线）

- **优点**：零代码量、零内存开销
- **缺点**：脱离 app；不满足「集成终端」需求；不承载 SSH 管理

### D. 自研 canvas 终端渲染器

- **优点**：理论最可控
- **缺点**：等于重造 xterm.js 的 scrollback、UTF-8、宽字符、终端协议层；高风险低收益

## 决策

**采用方案 A**：xterm.js + portable-pty + Tauri Channel 二进制流。

### 关键工程约束

1. **必须用 WebGL 渲染器**（`@xterm/addon-webgl`）—— 满足「不闪屏」
2. **必须用 `Channel<&[u8]>` 传二进制** —— 规避 Tauri JSON 事件序列化开销（已知 issue #9190 / #13405）
3. **Rust 侧 16ms 输出合并** —— 减少前端 writeBytes 调用次数
4. **scrollback 默认 10000 行** —— 内存可控
5. **`TerminalBackend` trait 抽象** —— v1 只实现 `LocalBackend`，v1.x 直接加 `SshBackend`（基于 russh），前端零改动

### 不采纳的理由

- B（libghostty）：等上游打 tag 再评估；当前 API 变动会拖垮我们的迭代节奏
- C（外部终端）：无法满足「在工具内运行」需求
- D（自研）：成本与风险不可接受

## 后续验证

- v1 上线后，监控内存使用（每个 tab idle 时 < 30MB 增量）
- v1.x 评估 SSH 后端时再起新 ADR

## 参考

- Tauri 官方 [Channel 文档](https://tauri.app/develop/calling-frontend/#channels)
- VS Code 终端架构：[vscode/src/vs/workbench/contrib/terminal](https://github.com/microsoft/vscode/tree/main/src/vs/workbench/contrib/terminal)
- xterm.js [WebGL addon](https://github.com/xtermjs/xterm.js/tree/master/addons/addon-webgl)
- portable-pty：[wezterm portable-pty](https://github.com/wez/wezterm/tree/main/portable-pty)