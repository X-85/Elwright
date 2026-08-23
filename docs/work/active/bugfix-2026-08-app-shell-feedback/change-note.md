# Bugfix · 应用壳布局 + 图标改造真机反馈

基线：`feat/chat` HEAD（`5197a9f`，含应用壳顶部图标导航改造 + v0.1.4 chat 阶段）。
分支：`bugfix/2026-08-app-shell-feedback`

## Bug 列表（用户逐条补充）

| # | 现象 | 复现路径 | 预期 | 根因（Agent 填） | 修复（Agent 填） | 状态 |
|---|------|---------|------|----------------|----------------|------|
| 1 | 终端打开后能看到光标，但敲键盘看不到任何回显/输出 | 打开终端 → 点「＋ 新建」→ 敲任意键 | shell prompt 与输入回显正常显示，命令可执行 | `terminal_open` 用 `Channel::new(no-op)` 自建 channel 并塞进返回值；PTY 输出的 `channel.send(bytes)` 只调用本地 no-op 闭包，从未跨 IPC。前端 bridge 传的 `channel` 参数被 Rust 签名忽略。输入其实已写进 PTY（`terminal_write` 链路通），但 shell 回显随输出流进 no-op——前端零输出，看似「不能输入」 | Rust：`terminal_open` 增加 `channel: Channel<Vec<u8>>` 命令参数（Tauri `CommandArg` 自动解析回 JS onmessage 回调），返回值只带 `u64` id，删除自建 no-op channel。前端注释同步说明返回值只有 id（调用处本就只用 id，无需改动） | 代码已修，待真机验证 |

## 根因链（Bug #1 详述）

Tauri 2.11.5 `ipc/channel.rs`：

- `Channel::new(closure)` 的 `send()` 只调本地闭包（channel.rs:296）——**不跨 IPC**。
- 只有作为命令参数反序列化的 `Channel`（`CommandArg::from_command`，channel.rs:300-316）经 `JavaScriptChannelId::channel_on(webview)` 拿到指向 JS `onmessage` 的 channel，`send()` 才 eval 前端回调。

v1 原始提交（562a1eb）即写反了方向，此后无人真机点过 GUI 清单（archive verification.md 中 GUI 段明确写「需要用户手点确认」，ROADMAP 却标了「真机验证通过」）。本次应用壳改造让用户第一次真打开了终端，暴露出来。

## 验证

每条修完后由用户在真机重走一遍复现路径，确认「预期」列行为出现。
