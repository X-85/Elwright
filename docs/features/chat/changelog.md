# AI 对话变更记录

## 2026-08-23（阶段② 会话管理）

- 新增桌面壳会话存储 `src-tauri/src/chat_store.rs`：`~/.elwright/chats/<id>.json` 一文件一会话；手写 UTC ISO8601 时间戳（字典序可排序）、`AtomicU64` 计数器生成 id，零新依赖；列表按 updated_at 降序、损坏文件跳过、删除幂等；6 个单元测试。
- 桌面壳新增 4 个 IPC：`chat_list_sessions` / `chat_load_session` / `chat_save_session` / `chat_delete_session`；`chat_save_session` 为 upsert 并保留原 created_at。
- 前端：`ChatView.vue` 改两栏布局——左侧会话侧栏（新建「＋」/切换/重命名 ✎ 或双击/删除 × 带 confirm），主区为原对话区；自动保存（用户发出与回复成功后），错误占位不入文件；标题默认取第一条用户消息（≤24 字符）；空会话不落盘。
- `Bridge` 新增会话四方法（snake_case↔camelCase 映射）；浏览器预览 list→空、save/delete 静默忽略，不模拟持久化。

## 2026-08-22（阶段① 对话基础实现）

- core `llm.rs`：新增 `ChatMessage` 与 `LlmClient::chat_messages`（多轮）；原 `chat(system, user)` 改为其封装，invoke 路径行为不变。新增 `CHAT_SYSTEM_PROMPT` 常量。
- 桌面壳新增 IPC `chat_completion`：复用 ConfigLayers 配置链，system 提示词由 Rust 侧前置（前端仅可传 user/assistant，注入 system 直接拒绝）；未配置/失败返回中文错误，不做降级 SOP。
- 前端：侧栏新增一级导航（能力工具箱 ⇄ AI 对话）；新增 `ChatView.vue`（多轮消息、生成中、停止=丢弃在途结果、失败重试、超长输入提示）；`Bridge` 新增 `chat()`（预览模式明确降级）。
- 模型输出按不可信 Markdown 渲染（ADR-002：覆写 html/link/image renderer，零新依赖），代码块带复制按钮，注入样本实测通过。
- 会话仅内存态；流式、请求级取消、本地会话管理分别属阶段②④，未实现。

## 2026-08-22

- 将 AI 对话登记为独立桌面 Feature，规划为四阶段：对话基础、会话管理、能力协作、流式与跨平台完善。
- 明确对话页不做隐式执行 Agent，能力调用必须经过用户确认。
