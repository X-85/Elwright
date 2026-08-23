# Plan：AI 对话 阶段①（对话基础）

## 目标

桌面壳新增「AI 对话」一级页面：多轮 user/assistant 文本对话、Markdown/代码块渲染与复制、发送/停止/重试、模型状态展示。未配置 LLM 时显示配置引导和离线能力提示，不显示空白聊天或通用错误。**非流式**（流式输出与真取消属阶段④）。

对齐 `docs/features/chat/README.md` 阶段 1 范围与 behavior.md「页面入口 / 消息与会话」两节。

## 技术方案

### core（src-tauri/src/core/）

- `llm.rs`：新增 `pub struct ChatMessage { role, content }` 与 `LlmClient::chat_messages(&self, &[ChatMessage]) -> Result<String, String>`；现有 `chat(system, user)`（invoke 路径在用）改为在其上封装，行为不变。请求构造、鉴权、60s 超时、响应解析复用现有代码。
- 系统提示词为 core 内常量，由 **Rust 侧前置**到 messages（前端只传 user/assistant 历史，用户输入无法覆盖 system——behavior.md 的安全要求）。

### 桌面壳 IPC（main.rs）

- 新命令 `chat_completion(messages: Vec<ChatMessageArg>) -> Result<String, String>`：
  - 配置走 `ConfigLayers`（与 `invoke_skill` 同一合并链路）；`base_url` 未配置 → 中文 Err，前端据此显示配置引导。
  - 请求失败 → 中文 Err 原样返回；**对话无降级 SOP 概念**，会话保留由前端负责（behavior.md：请求失败保留会话并提供重试）。
  - async 命令 + `spawn_blocking` 跑 blocking 请求（与现有 `invoke_skill` 模式一致，不阻塞主线程）。

### 前端（src/）

- `lib/bridge.ts`：`Bridge` 新增 `chat(messages: { role: 'user' | 'assistant'; content: string }[]): Promise<string>`；tauri 桥 invoke `chat_completion`；浏览器桥抛「【预览模式】」明确提示（不模拟对话，与 getLlmConfig 同口径）。
- `App.vue`：顶层视图切换（能力工具箱 ⇄ AI 对话），AI 对话为一级入口，可返回工具箱。
- 新组件 `components/ChatView.vue`：
  - 消息列表（user/assistant 气泡）、生成中状态、失败重试（重发原始输入）、多行输入（Enter 发送 / Shift+Enter 换行）。
  - 模型状态条：复用 `getLlmConfig()`（model + 每字段来源 + key 打码）；未配置 → 配置引导 + 打开 ⚙ 模型设置入口。
  - Markdown 渲染：**先转义 `<` 再走 marked**——模型输出是不可信文本，AGENTS.md 约定 marked 直出 v-html 仅限可信本地文件，本项需新增 **ADR-002**（转义后解析，零新依赖）。代码块带复制按钮。
  - 停止（阶段①口径）：前端丢弃在途结果并立即恢复输入；Rust 侧请求照常完成但结果不入列。真正的请求级取消（请求 id + 中断）与流式一起在阶段④实现。
  - 会话仅内存态（刷新即失），持久化与多会话管理属阶段②。

## 实现步骤

1. core：`ChatMessage` + `chat_messages` + 单测（多轮 role 序列化、`chat()` 封装行为不变）。
2. `main.rs`：`chat_completion` 命令；CI mock LLM 冒烟新增 chat 回归（复用现有 mock_llm.py 模式）。
3. `bridge.ts`：接口 + tauri/browser 双桥实现。
4. `ChatView.vue` + `App.vue` 视图切换 + 模型状态条。
5. Markdown 渲染方案落地 + 写 ADR-002 + 恶意样本验证（含 `<script>`、`<img onerror>` 的回复不执行）。
6. 更新 `docs/features/chat/`：README「当前状态」、changelog 补阶段①条目，behavior/architecture 按实际实现校正。
7. 真机端到端验证（见 verification 清单）。

## 非目标

- 会话持久化/新建/切换/重命名/删除（阶段②）；能力协作与推荐卡片（阶段③，ADR-001 已定显式确认原则）；流式输出、请求级取消、上下文长度治理与跨平台完善（阶段④）。
- 不做 token 计数与自动截断；超长输入仅轻量提示。
- CLI 不加 chat 子命令（对话定位为桌面入口；确需时另行立项）。

## 风险与验证

- **不可信 Markdown 渲染**：核心风险。方案为 escape-before-marked（零新依赖）；上线前用注入样本实测。
- **blocking 请求最长 60s**：UI 必须全程可交互（停止/切页不卡）；async 命令已隔离主线程。
- 前端契约变更：`Bridge` 接口新增方法，两个适配器都必须实现（TS 编译期强制）。
- 真机验证清单：真实端点多轮对话（发送→回复→追问）、停止丢弃、失败重试、未配置引导态、⚙ 设置后返回对话页状态刷新。
