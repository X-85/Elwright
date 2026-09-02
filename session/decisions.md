# 会话决定

## D1：自动化交互测试分层

- 已确认：Elwright 的按钮与页面交互优先由 Playwright 浏览器端到端测试验证；Rust 单元测试和 Vitest 继续覆盖核心、Bridge 与纯逻辑。
- 未验证：Tauri 真正的原生文件选择、软件启动和终端 PTY 行为需要独立桌面壳冒烟测试，不能由浏览器预览替代。
- 已验证：浏览器冒烟测试使用独立上下文和清空后的 localStorage，资源仅保存 `virtual://` URI 字符串，不访问真实文件或 `~/.elwright/`。
