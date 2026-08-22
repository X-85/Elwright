# 集成终端 checklist

- [ ] feature 文档骨架（README/behavior/architecture/changelog/ADR-001）
- [ ] Cargo.toml 加 portable-pty
- [ ] core/terminal/mod.rs + backend.rs + local.rs + ipc.rs
- [ ] TerminalBackend trait 单测（mock backend）
- [ ] SessionRegistry + 输出合并 16ms 单测
- [ ] Tauri IPC 命令注册（open/write/resize/close）
- [ ] Exit 钩子 kill all
- [ ] package.json 加 xterm + addon-webgl + addon-fit
- [ ] lib/bridge.ts TerminalSession + 方法
- [ ] TerminalPanel.vue（抽屉 + tab 管理）
- [ ] TerminalView.vue（xterm 挂载、WebGL、resize observer）
- [ ] App.vue 挂载 TerminalPanel
- [ ] CapabilityDetail.vue「在终端中运行」按钮
- [ ] 端到端：macOS 本地 shell 跑通
- [ ] 端到端：Windows pwsh 跑通（公司机）
- [ ] `cargo test` 全绿
- [ ] 前端 `npm run build` 通过