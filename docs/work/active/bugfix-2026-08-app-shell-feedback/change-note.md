# Bugfix · 应用壳布局 + 图标改造真机反馈

基线：`feat/chat` HEAD（`5197a9f`，含应用壳顶部图标导航改造 + v0.1.4 chat 阶段）。
分支：`bugfix/2026-08-app-shell-feedback`

## Bug 列表（用户逐条补充）

| # | 现象 | 复现路径 | 预期 | 根因（Agent 填） | 修复（Agent 填） | 状态 |
|---|------|---------|------|----------------|----------------|------|
| 1 | 终端打开后能看到光标，但敲键盘看不到任何回显/输出 | 打开终端 → 点「＋ 新建」→ 敲任意键 | shell prompt 与输入回显正常显示，命令可执行 | `terminal_open` 用 `Channel::new(no-op)` 自建 channel 并塞进返回值；PTY 输出的 `channel.send(bytes)` 只调用本地 no-op 闭包，从未跨 IPC。前端 bridge 传的 `channel` 参数被 Rust 签名忽略。输入其实已写进 PTY（`terminal_write` 链路通），但 shell 回显随输出流进 no-op——前端零输出，看似「不能输入」 | Rust：`terminal_open` 增加 `channel: Channel<Vec<u8>>` 命令参数（Tauri `CommandArg` 自动解析回 JS onmessage 回调），返回值只带 `u64` id，删除自建 no-op channel。前端注释同步说明返回值只有 id（调用处本就只用 id，无需改动） | 代码已修，待真机验证 |

## 终端交互优化（2026-08-23 第二批，参考 ZCode UI）

| # | 需求 | 实现 |
|---|------|------|
| O1 | 顶栏点终端按钮：下方直接弹出并新建一个终端，进入主目录 | App.vue `toggleTerminal` → TerminalPanel 新增 `toggleFromToolbar()`：无 tab 时 `openTab()`（cwd=主目录）并展开，有 tab 纯切换展开态。Bridge 新增 `homeDir()`（Tauri `@tauri-apps/api/path`，浏览器返回 null）；openTab 默认 cwd 从 `props.cwd`（应用目录）改为 homeDir，「在终端中运行」仍显式传应用 cwd（`ew run` 靠 cwd 上溯找注册表根） |
| O2 | 新建按钮「＋ 新建」改「＋」；关闭键 × | 模板文案改「＋」（title 提示保留"新建终端标签"），关闭键本就是 ×，tab label 的 `@mousedown.stop` 防拖拽误触 |
| O3 | 终端窗口允许往上拉 | 表头整体 `cursor: ns-resize` + `mousedown` 拖拽：`heightPct` 20–85vh 夹紧，拖动时全局 cursor/禁选中文本；移除旧 4px 小把手（`.resize-handle`） |
| O4 | 左侧 ▼ toggle 移到右侧并替换为 × | 表头改为 `[＋] [tabs…] [×]`：右侧 × = 收起面板（会话保留，再点顶栏终端按钮恢复）；tab 自身 × 仍是关闭该会话。展开后无 tab 时收起路径只剩顶栏按钮（面板 header 随面板隐藏） |

改动文件：`src/lib/bridge.ts`（homeDir 接口 + 双实现）、`src/components/TerminalPanel.vue`（toggleFromToolbar / 拖拽 / 按钮文案）、`src/App.vue`（顶栏按钮接新方法）。

## 根因链（Bug #1 详述）

Tauri 2.11.5 `ipc/channel.rs`：

- `Channel::new(closure)` 的 `send()` 只调本地闭包（channel.rs:296）——**不跨 IPC**。
- 只有作为命令参数反序列化的 `Channel`（`CommandArg::from_command`，channel.rs:300-316）经 `JavaScriptChannelId::channel_on(webview)` 拿到指向 JS `onmessage` 的 channel，`send()` 才 eval 前端回调。

v1 原始提交（562a1eb）即写反了方向，此后无人真机点过 GUI 清单（archive verification.md 中 GUI 段明确写「需要用户手点确认」，ROADMAP 却标了「真机验证通过」）。本次应用壳改造让用户第一次真打开了终端，暴露出来。

## 验证

每条修完后由用户在真机重走一遍复现路径，确认「预期」列行为出现。
