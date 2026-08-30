# ADR-001 — e2e 选型：弃 tauri-driver，IPC mock + Playwright 分层

- 状态：已采纳（2026-08-24）
- 关联任务：`docs/work/active/enhancement-2026-08-quality-tier2-e2e/`

## 背景

ROADMAP 工程质量第二档要求「e2e 冒烟抓集成断线类 bug」。候选路线三条：

1. **tauri-driver**（官方 WebDriver）驱动打包后的桌面壳
2. **tauri mock runtime**（`tauri::test`）在进程内以真实 IPC 协议调用命令
3. **Playwright** 驱动浏览器预览壳（vite dev + `/api/*` 只读端点）

终端两个 bug 的教训：Bug #1（`terminal_open` 的 Channel 参数被自建 no-op 替换）出在 **IPC 协议接缝**，Bug #2 出在会话状态层。要防的是「各层单测全绿但接缝断线」。

## 决策

**弃 tauri-driver，采用路线 2 + 3 分层：**

- **IPC 层**（`src-tauri/tests/terminal_ipc.rs`）：mock runtime + `get_ipc_response`。理由：
  - 走完整 `CommandArg` 解析路径，Channel 参数按 `__CHANNEL__:N` 传入——正是 Bug #1 的出错路径，tauri-driver 反而测不到（它测 UI 渲染，不测参数解析）。
  - macOS/Linux 可用真 `LocalBackend` 跑真 PTY 全链路（open → write echo/exit → 断言 write 报错），无需任何 GUI 驱动。
  - 随现有 rust matrix job 三平台跑，CI 零新增 job。
- **浏览器层**（`src/e2e/`）：Playwright chromium。理由：
  - 覆盖 browserBridge ↔ dev 插件接缝与预览模式降级守卫（终端按钮不渲染、【预览模式】文案），这是 mock runtime 看不到的前端行为。
  - 只加 chromium 一个浏览器，CI frontend job 加一步即可。

**弃 tauri-driver 的原因：**

- Windows CI 无 WinAppDriver（tauri-driver 在 Windows 的硬依赖），三平台矩阵立刻缺一角。
- Linux 侧需要 xvfb 且有窗口焦点前科，稳定性投入不成比例。
- 它能覆盖的「桌面壳 UI 真实渲染」恰恰不是本轮 bug 的出事层；IPC 协议接缝它走不进去。

## 后果

- 桌面壳「WebView 内 UI 真实渲染」仍是空白层，唯一【手测】项（`tauri dev` 冒烟）覆盖它；若未来该层出 bug，再评估 tauri-driver 或人工脚本。
- Windows+CI 跳过真 PTY 用例（ConPTY 无交互服务会话挂起前科，`bugfix-2026-08-win-ci-pty-hang`），协议用例照跑；本机 Windows 不设 `CI` 变量仍执行真 PTY。
- mock runtime 的 `last_evaluated_script` 是覆盖式存储（多帧断言不可靠），channel 输出观测用可观测后端（EchoBackend）行为断言替代，见测试文件头注释。
