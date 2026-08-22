# 集成终端 v1（feature-2026-08-integrated-terminal）

## 目标

Elwright 桌面壳内嵌一个集成终端（底部抽屉 + 多 tab），支持本地 shell 与 script capability「在终端中运行」联动。SSH 后端留待 v1.x 通过 trait 扩展。

## 范围

### 必做（v1）

- Rust 侧 `TerminalBackend` trait + `LocalBackend`（portable-pty）实现
- Tauri IPC：`open_terminal`（返回 id + Channel）、`terminal_write`、`terminal_resize`、`terminal_close`
- PTY 输出按 ~16ms 合并、通过 `Channel<&[u8]>` 二进制流回前端
- 前端 `TerminalPanel.vue`（底部抽屉 + 多 tab）与 `TerminalView.vue`（单 tab 渲染）
- xterm.js + `@xterm/addon-webgl` + `@xterm/addon-fit`
- 退出应用时自动 kill 所有 session
- script capability 加「在终端中运行」按钮（新 tab + 自动执行命令）

### 不做（v1）

- SSH / 远端会话（v1.x 计划）
- 分屏、自定义主题、命令历史持久化、搜索
- 自定义字体/字号/配色（v2）

## 实现步骤

1. Cargo.toml 加 `portable-pty = "0.8"`；package.json 加 `xterm` / `@xterm/addon-webgl` / `@xterm/addon-fit`
2. 写 `core/terminal/{mod.rs, backend.rs, local.rs, ipc.rs}`
3. main.rs 注册 IPC 命令 + Exit 钩子 + 平台默认 shell 探测
4. bridge.ts 加 `TerminalSession` 接口与方法
5. 写 `TerminalPanel.vue` + `TerminalView.vue`
6. App.vue 挂载面板
7. CapabilityDetail.vue 加「在终端中运行」按钮
8. 单元测试：mock backend、16ms 合并、session 注册表
9. 端到端手动验证（macOS / Windows 各一次）

## 风险

- portable-pty Windows ConPTY 兼容性（旧 Win10 缺失）—— 启动探测，缺则提示升级
- WebGL 上下文丢失 —— xterm.js 自动 fallback DOM
- xterm.js dispose 内存泄漏 —— 严格 dispose 顺序
- PTY 子进程僵尸 —— 双保险 SIGTERM/SIGKILL

## 验证方式

- `cargo test` 全部通过
- 前端 `npm run build` 通过
- 端到端：本地 zsh / pwsh，WebGL 渲染启用（DevTools 验证），resize 工作
- 「在终端中运行」端到端：点 button → 新 tab → 脚本命令可见输出