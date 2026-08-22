# Checklist

- [x] core：`ChatMessage` + `LlmClient::chat_messages` + `chat()` 改封装，单测过
- [x] main.rs：`chat_completion` IPC 命令（ConfigLayers 链路 + Rust 侧前置 system 提示词）
- [x] mock 回归：cargo 单测内置 mock 端点（TcpListener）覆盖多轮/鉴权/封装路径；CI 现有 mock invoke 冒烟继续覆盖 chat() 重构链路
- [x] bridge.ts：`chat()` 接口 + tauri/browser 双桥
- [x] App.vue：工具箱 ⇄ AI 对话视图切换（一级入口，⚙ 关闭后刷新对话页配置状态）
- [x] ChatView.vue：消息列表 / 生成中 / 停止（丢弃在途结果）/ 失败重试 / 多行输入（Enter/Shift+Enter/中文输入法合成态）
- [x] 模型状态条 + 未配置引导（接 ⚙ 模型设置）+ 预览模式降级提示
- [x] Markdown 渲染：renderer 覆写（ADR-002 方案，代码块保真）+ 代码块复制 + 注入样本实测
- [x] ADR-002（不可信 Markdown 渲染方案）入 `docs/features/chat/decisions/`
- [x] chat Feature 文档更新（README 当前状态 / changelog / behavior·architecture 校正）
- [x] 自动化验证（cargo test 31 过 / 前端 build / 注入样本 / CLI mock 端到端）——见 verification.md
- [x] 浏览器预览模式自动化验证（视图切换/预览降级/错误气泡/重试/工具箱回切）
- [x] macOS 桌面包构建（Elwright.app 产出）+ 启动自检（mock LLM 不崩溃）
- [ ] 原生窗口 GUI 交互（多轮对话/停止/重试/未配置引导/⚙ 刷新/代码块复制）——IAB 无法驱动 WKWebView，需人在桌面 app 操作
