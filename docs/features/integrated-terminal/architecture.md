# 架构（Architecture）

## 整体结构

```
┌─────────────────── 前端 (Vue) ───────────────────┐
│  TerminalPanel.vue  (底部抽屉, 多 tab)            │
│  ├─ TerminalView.vue (单 tab, xterm.js + WebGL)  │
│  └─ lib/bridge.ts   (TerminalSession 接口)        │
└────────────────────┬─────────────────────────────┘
                     │  Tauri IPC
                     ▼
┌─────────────────── Rust (Tauri 壳) ──────────────┐
│  main.rs                                          │
│  ├─ terminal_open → Core::open_session            │
│  ├─ terminal_write (事件/请求)                    │
│  ├─ terminal_resize                               │
│  ├─ terminal_close                                │
│  └─ Channel<&[u8]>  ← 二进制流回传                │
└────────────────────┬─────────────────────────────┘
                     │
                     ▼
┌─────────────────── core/terminal ─────────────────┐
│  mod.rs       : TerminalBackend trait            │
│  local.rs     : LocalBackend (portable-pty)      │
│  ipc.rs       : SessionRegistry, PTY→Channel     │
│  (ssh.rs      : v1.x 预留)                       │
└──────────────────────────────────────────────────┘
```

## 核心抽象：TerminalBackend trait

```rust
pub trait TerminalBackend: Send + Sync {
    fn spawn(
        &self,
        shell: &str,
        cwd: &Path,
        cols: u16,
        rows: u16,
        env: &[(String, String)],
    ) -> Result<TerminalHandle, String>;

    fn write(&self, id: SessionId, bytes: &[u8]) -> Result<(), String>;
    fn resize(&self, id: SessionId, cols: u16, rows: u16) -> Result<(), String>;
    fn kill(&self, id: SessionId) -> Result<(), String>;
}

pub struct TerminalHandle {
    pub reader: Box<dyn AsyncRead + Send + Unpin>, // 输出流
    pub writer: Box<dyn AsyncWrite + Send + Unpin>, // 输入流（PTY master 写端）
    pub killer: Box<dyn FnOnce() + Send>,          // 关闭回调
}
```

为什么这样设计：

- **SSH 后端**（v1.x）可实现同一 trait，用 `russh_channel` 拿到 `AsyncRead/AsyncWrite`；前端零改动
- **测试**可以用一个 in-memory backend（双向 pipe），单测 Rust 端读写与输出合并逻辑，无需真实 PTY

## 数据流细节

### 子进程输出 → 前端

1. `portable-pty` 在 spawn 时返回 master/slave pair，slave 绑定给子进程
2. Rust 端把 master 的 reader 转成 `tokio::process::ChildStdout` 风格的 `AsyncRead`
3. 每个 session 起一个 `tokio::spawn`：
   - 按 ~16ms 时间窗或 4KB 缓冲合并数据
   - 调用 `channel.send(bytes).await` 把字节流送前端
   - 遇到 EOF（子进程退出）→ 关闭 channel，session 标记为 exited
4. 前端订阅 `channel`，收到字节直接 `term.writeBytes(data)`（避免 UTF-8 decode 损耗）

### 用户按键 → 子进程

1. `xterm.js` `onData` 回调（每个 tab 独立）
2. `invoke('terminal_write', { id, data: base64(bytes) })`
3. Rust 端 decode bytes → `backend.write(id, &bytes)`
4. 写到 PTY master

### Resize

- 前端用 `FitAddon` 监听容器尺寸变化 + `requestAnimationFrame` 节流
- 把 `(cols, rows)` 通过 IPC 推给 Rust
- Rust 调用 `portable-pty::master.resize(cols, rows)`（关键，TUI 程序靠这个信号重绘）

## Tauri Channel 用法（避坑要点）

为什么用 `Channel<&[u8]>` 而不是 `emit` 事件：

- Tauri 事件系统只支持 JSON，序列化开销大、流式体验差（官方 issue #9190, #13405）
- `Channel<&[u8]>` 直接传二进制，适合 PTY 原始字节
- `Channel` 也支持 backpressure（前端读取慢时 Rust 自动阻塞）

Rust 侧注册 channel（伪代码）：

```rust
#[tauri::command]
async fn terminal_open(
    app: AppHandle,
    state: State<'_, SessionRegistry>,
) -> Result<(SessionId, Channel<Vec<u8>>), String> {
    let channel = Channel::new(app.clone());
    let id = state.spawn_local(&channel)?;
    Ok((id, channel))
}
```

前端：

```ts
const channel = new Channel<number[]>({ /* ... */ })
const id = await invoke<number>('terminal_open')
channel.onmessage = (bytes) => term.writeBytes(new Uint8Array(bytes))
```

## 进程生命周期

- 应用退出：`AppHandle` 的 `RunEvent::Exit` → `SessionRegistry::kill_all()`
- 单 tab 关闭：前端 `invoke('terminal_close', { id })` → registry remove + killer 调用
- 父子进程：portable-pty 默认会清理子进程（master drop 时 close slave），但为保险显式 killer 用 `nix::sys::signal` 或 Windows `TerminateProcess`

## 与现有功能的集成

- **script capability「在终端中运行」**：前端在 CapabilityDetail 加按钮，调用 `openTerminal({ name: cap.name, cwd: ... })` 后立即 `terminal_write` 写命令字符串 + `\n`
- **IPC 命名空间**：所有 terminal 相关 IPC 加 `terminal_` 前缀，避免与现有命令冲突
- **日志/错误**：用户态错误走 toast（前端），Rust 内部错误用 `tracing` 记录（v1.x 接入）

## 文件清单（v1 增量）

新增：
- `src-tauri/src/core/terminal/mod.rs`
- `src-tauri/src/core/terminal/backend.rs`（trait）
- `src-tauri/src/core/terminal/local.rs`（portable-pty 实现）
- `src-tauri/src/core/terminal/ipc.rs`（SessionRegistry + Channel 整合）
- `src/components/TerminalPanel.vue`
- `src/components/TerminalView.vue`
- 单测若干（backend trait 的 mock 实现 + 输出合并逻辑）

修改：
- `src-tauri/Cargo.toml`（新增 portable-pty）
- `src-tauri/src/main.rs`（注册 IPC 命令、退出钩子）
- `src/package.json`（xterm.js + addon-webgl + addon-fit）
- `src/lib/bridge.ts`（TerminalSession 接口、bridge 方法）
- `src/App.vue`（挂载 TerminalPanel）
- `src/components/CapabilityDetail.vue`（加「在终端中运行」按钮）

## 测试策略

- **单元**：
  - `backend` trait 的 mock 实现，验证 `SessionRegistry` 输出合并逻辑
  - 16ms 合并：写 3 个短 burst → 期望一次性 flush
  - backpressure：写入端阻塞 → channel 不会无限 buffer
- **集成**（手动）：
  - macOS / Windows 各跑一次端到端（spawn zsh / pwsh，跑 `echo hello`、`tput cols` 之类）
  - 验证 WebGL 渲染（DevTools 看 canvas 是否启用）
  - 验证 resize（启 `top` 类 TUI 应用，拉抽屉）
- **CI**：
  - `cargo test`（单元）
  - 前端 `npm run build`
  - Tauri 端到端冒烟（v1.x 用 webdriver；v1 仅构建冒烟）

## 风险与缓解

| 风险 | 缓解 |
|---|---|
| portable-pty Windows ConPTY 兼容（Win10 旧版缺失） | 启动时探测 win 版本，< Win10 1809 提示用户升级 |
| WebGL 上下文在某些显卡/驱动下丢失 | 自动 fallback DOM（xterm addon-canvas 已被官方移除，不再依赖） |
| xterm.js 内存泄漏（已知 dispose bug） | 严格 dispose 顺序：fit → webgl → xterm → terminalDiv.remove() |
| PTY 子进程僵尸（罕见） | killer 用双重保险（SIGTERM → 3s 后 SIGKILL） |
| Channel 跨平台字节序差异 | 统一用 `Vec<u8>`，前端 `Uint8Array` 还原 |