# AI 对话变更记录

## 2026-08-22（阶段① 对话基础实现）

- core `llm.rs`：新增 `ChatMessage` 与 `LlmClient::chat_messages`（多轮）；原 `chat(system, user)` 改为其封装，invoke 路径行为不变。新增 `CHAT_SYSTEM_PROMPT` 常量。
- 桌面壳新增 IPC `chat_completion`：复用 ConfigLayers 配置链，system 提示词由 Rust 侧前置（前端仅可传 user/assistant，注入 system 直接拒绝）；未配置/失败返回中文错误，不做降级 SOP。
- 前端：侧栏新增一级导航（能力工具箱 ⇄ AI 对话）；新增 `ChatView.vue`（多轮消息、生成中、停止=丢弃在途结果、失败重试、超长输入提示）；`Bridge` 新增 `chat()`（预览模式明确降级）。
- 模型输出按不可信 Markdown 渲染（ADR-002：覆写 html/link/image renderer，零新依赖），代码块带复制按钮，注入样本实测通过。
- 会话仅内存态；流式、请求级取消、本地会话管理分别属阶段②④，未实现。

## 2026-08-22

- 将 AI 对话登记为独立桌面 Feature，规划为四阶段：对话基础、会话管理、能力协作、流式与跨平台完善。
- 明确对话页不做隐式执行 Agent，能力调用必须经过用户确认。
