# 验证记录

## 单元测试

```
cargo test --lib
running 29 tests
…
test result: ok. 29 passed; 0 failed; 0 ignored
```

新增覆盖：
- `core::terminal::local::tests::spawn_produces_expected_output` — 真实 PTY spawn，能拿到 echo 输出（hello/world）
- `core::terminal::ipc::tests::registry_keeps_sessions_independent` — mock backend 验证多 session 隔离、kill/kill_all 正确性
- `core::terminal::ipc::tests::write_to_unknown_id_errors` — 未知 id 返回友好中文错误

## 前端构建

```
npm install   # added 3 packages（@xterm/xterm + addon-webgl + addon-fit）
npm run build
✓ built in 134ms
dist/index.html                    0.42 kB
dist/assets/index-…css            11.52 kB
dist/assets/index-…js            524.45 kB
```

xterm.js + addons 整包约 500kB，符合预期（VS Code 同样量级；后续可用 dynamic import 拆 chunk）。

## 编译

- `cargo check --bin elwright`：通过（4 个 snake_case 警告为 IPC camelCase 参数，按 Tauri 约定保留）
- `cargo build --bin ew`：通过
- `cargo test --lib`：29 个测试全绿

## 端到端（待真机跑）

> macOS / Windows 桌面端真机验证清单（用户在自己机器上跑）：
> 1. `cd src && npm run dev` 起 vite，桌面应用实际启动
> 2. 点底部 ▲ 展开抽屉 → 点「＋ 新建」开第一个 terminal
> 3. 跑 `echo hello`，验证输出及时出现、无闪屏（DevTools 看 canvas 而非 div）
> 4. 开第二个 tab，输入 `top` 或 `htop`，拖拽面板高度 → TUI 重绘
> 5. CapabilityDetail 选一个 script → 点「⌨ 在终端中运行」→ 新 tab 自动跑 `ew run <id>`
> 6. 关闭应用主窗口 → 所有 tab 被 kill（无残留进程）

## 已知限制（v1）

- Windows 用户需 Win10 1809+ 才有 ConPTY（portable-pty 自动选；旧版无解）
- WebGL 在某些远程桌面/集显下会丢上下文，xterm.js 自动 fallback DOM（控制台 warn，用户无感）
- 当前 spawn 后 150ms 才发命令字符串（heuristic），CLI 启动慢的机器可能错过 prompt；后续接 PTY ready 事件优化