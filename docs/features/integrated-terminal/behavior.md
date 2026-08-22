# 行为（Behavior）

## v1 当前规则

### 终端面板

- 桌面壳底部抽屉布局：高度可在 0–80% 视口高度间拖动；最小化时只露一行标题栏（标签 + 高度/关闭按钮）。
- **多 tab**：同一面板内可开多个标签（每个标签 = 一个独立终端会话）。
- 标签可重命名（双击或右键菜单）、拖拽排序。
- 关闭最后一个标签 → 面板自动最小化（不销毁，用户可一键展开恢复）。
- 应用窗口隐藏/最小化时，终端进程持续运行；退出应用 → 自动 kill 所有会话。

### 会话生命周期

- 每个标签 = 一个 `TerminalSession`，由 Rust 端 `TerminalBackend` 提供：
  - `spawn(shell, cwd, cols, rows)` → 返回 session id + reader/writer 双工通道
  - `write(session_id, bytes)` → 把前端按键写入 PTY master
  - `resize(session_id, cols, rows)` → 通知 PTY 窗口大小变化（用于 TUI 程序正确重绘）
  - `kill(session_id)` → 关闭 PTY、回收子进程
- v1 默认 shell：
  - macOS / Linux：`$SHELL`，回退 `/bin/zsh`（macOS）/ `/bin/bash`（Linux）
  - Windows：`pwsh`（探测 PowerShell 7），回退 `powershell`（Windows PowerShell 5.1）
- v1 默认 cwd：当前桌面应用的工作目录（可后续让用户配置）

### 数据流

- **PTY 输出**（子进程 → 终端显示）：
  - Rust 侧起一个 spawn 线程持续 read PTY master，按 ~16ms 窗口合并输出
  - 通过 Tauri `Channel<&[u8]>` 以 **二进制字节流** 把数据推到前端（绝不走 JSON 事件）
  - 前端 `xterm.js` 写入到对应 `Terminal` 实例
- **用户输入**（键盘 → 子进程）：
  - 前端 `xterm.js` `onData` 事件 → 通过 `terminalWrite` IPC → Rust → `write_all` 到 PTY master
- **resize**：
  - 前端监听 `xterm.js` 的 `proposeGeometry` 或 ResizeObserver → `terminalResize` IPC → Rust

### 渲染器与体验

- 默认启用 **WebGL renderer**（`@xterm/addon-webgl`，GPU 绘制，无 DOM reflow 闪屏）
- WebGL 上下文丢失 → 自动回退 DOM 渲染器（VS Code 同款降级）
- scrollback 默认 10000 行，可在终端右键菜单切换（1000 / 10000 / 100000 / 无限）
- 字体使用系统等宽字体；字号可在终端设置中调整（v1.x）

### 「在终端中运行」能力联动（v1）

- script 类型 capability 在 CapabilityDetail 上多一个按钮：**「在终端中运行」**
- 点击行为：新建一个终端 tab（不聚焦已有），自动执行对应脚本的命令（如 `python3 /path/to/script.py`），用户可看到完整运行过程、错误信息、可继续输入
- 与「运行」按钮（捕获 stdout 后一次性返回结果）并存，按需选择

### 不允许 / 边界

- 不实现 SSH / 远端会话（v1.x 计划，通过新增 `SshBackend: TerminalBackend`）
- 不实现分屏（split pane）（v2 评估）
- 不实现自定义主题（跟随 xterm 默认主题，v2 评估深色/浅色）
- 不保存历史命令持久化（每次会话独立）

### 失败与降级

- PTY 启动失败（解释器不存在等）：终端显示红色错误行 + toast 提示中文原因
- 子进程退出非 0：tab 标题加 `[exit N]`，PTY 关闭但不自动销毁 tab（用户可看历史输出）
- WebGL 不可用：自动 fallback DOM，xterm.js 控制台 warn，不暴露给用户

## 后续版本演进

| 版本 | 内容 |
|---|---|
| v1.x | SSH 后端（russh）实现 `TerminalBackend` trait；xshell-like 会话管理（保存主机、密钥、跳板） |
| v2 | 分屏、主题、命令持久化历史、搜索 |