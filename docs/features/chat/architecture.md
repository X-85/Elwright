# AI 对话架构

## 调用关系

```text
ChatView.vue
    ↓ ChatBridge / Bridge
Tauri IPC：chat、stream、cancel、session storage
    ↓
core::llm（多轮 OpenAI-compatible 请求）
    ↓
用户配置端点 / 本地模型

能力确认卡片 ─→ 现有 Bridge ─→ registry / executor / invoke / terminal
```

## 分阶段技术方案

- 阶段 1（已实现 2026-08-22）：`llm.rs` 单轮 `system/user` 请求已扩展为受控 `ChatMessage[]`（`chat_messages`，原 `chat()` 改为其封装），非流式返回；IPC `chat_completion` 与 Bridge `chat()` 已接入。system 提示词（`CHAT_SYSTEM_PROMPT`）由 Rust 侧前置，IPC 层拒绝 user/assistant 之外的角色。停止为前端序号丢弃（请求级取消属阶段④）。模型输出渲染走 `src/lib/safeMarkdown.ts`（ADR-002）。
- 阶段 2：桌面壳增加会话文件存储和生命周期管理，存放在 `~/.elwright/chats/` 或等价用户目录；API Key 永不进入会话模型。
- 阶段 3：增加能力协作协议。模型推荐结果先解析为展示模型，用户确认后才转调用现有能力接口；不把任意模型输出当作可执行指令。
- 阶段 4：用 Tauri Channel 推送增量文本，增加请求 id、取消、超时、上下文长度提示和跨平台验证。

## 边界与风险

- `src-tauri/src/core/` 只负责通用 LLM 请求模型和结果解析，不放 Vue 页面、会话 UI 或桌面窗口逻辑。
- 会话存储属于桌面壳；浏览器预览只能展示未连接或降级状态，不模拟真实桌面持久化。
- 多轮上下文可能包含敏感信息和超出模型限制，需提供清空上下文和历史删除，并在发送前做长度控制。
- 流式输出必须能停止并回收请求，应用退出时不得留下后台请求或泄露输出。
