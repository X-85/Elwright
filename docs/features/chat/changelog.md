# AI 对话变更记录

## 2026-08-31（阶段④ 余项：长上下文，ADR-004）

- 新增 `core::chat_context::fit_messages`：字符预算滑动窗口（默认 24000 字符，
  `contextBudgetChars` 配置链字段 / `ELWRIGHT_LLM_CONTEXT_BUDGET_CHARS` 环境变量可覆盖）。
  最新消息必留（超预算中段截断留头尾并标注），更早消息从新到旧整条保留、放不下整条丢弃。
- `assemble_chat_messages` 收口 `chat_completion` 与 `chat_completion_stream`，
  静默裁剪，system 不受预算影响。
- `ew config` 新增 context 行展示；`ew config set` 新增 `context_budget_chars` 键。
- 顺手修数据丢失隐患：`ew config set` 原按纯 flat 表回写 config.json，配置含
  profiles/activeProfile（Q19 档案）时会被整体抹掉——改经 `UserConfigFile` 读写保留档案。
- 测试：`chat_context` 6 单测 + 新 `tests/chat_completion_ipc.rs`（本地 mock LLM
  服务捕获真实请求体，断言 system 前置 / 最新 user 完整 / 历史收敛 ≤ 预算 / 最旧轮次被裁）。
- 决策与拒绝项见 `decisions/ADR-004-long-context.md`。

## 2026-08-23（阶段② 会话管理）

- 新增桌面壳会话存储 `src-tauri/src/chat_store.rs`：`~/.elwright/chats/<id>.json` 一文件一会话；手写 UTC ISO8601 时间戳（字典序可排序）、`AtomicU64` 计数器生成 id，零新依赖；列表按 updated_at 降序、损坏文件跳过、删除幂等；6 个单元测试。
- 桌面壳新增 4 个 IPC：`chat_list_sessions` / `chat_load_session` / `chat_save_session` / `chat_delete_session`；`chat_save_session` 为 upsert 并保留原 created_at。
- 前端：`ChatView.vue` 改两栏布局——左侧会话侧栏（新建「＋」/切换/重命名 ✎ 或双击/删除 × 带 confirm），主区为原对话区；自动保存（用户发出与回复成功后），错误占位不入文件；标题默认取第一条用户消息（≤24 字符）；空会话不落盘。
- `Bridge` 新增会话四方法（snake_case↔camelCase 映射）；浏览器预览 list→空、save/delete 静默忽略，不模拟持久化。

## 2026-08-31（阶段③ 能力协作 + 阶段④ 流式/取消）

- 阶段③ 能力协作（v0.1.10 PR #6）：`chat_propose_capability` IPC + `Bridge.chatProposeCapability`；`CapabilityProposal` / `CapabilityProposalResult` JSON；ChatView 收到 `\`\`\`json-proposal` 围栏后渲染「能力提议/调用确认卡片」，含运行参数输入、确认运行、错误回灌；执行链路复用 `core::invoke::invoke_capability`，未配置 LLM 也走离线 SOP。
- 阶段④ 流式与请求级取消（v0.1.10 PR #8）：`chat_stream_start` / `chat_stream_event` / `chat_stream_cancel` 三个 IPC，事件通过 Tauri channel 流式推送 `chunk` / `done` / `error`；请求 id 由后端生成、取消按 id 命中；`core::llm` 暴露 `chat_messages_stream`，复用同一配置链与 system 提示词；前端 `ChatView.vue` 改为 SSE-style 增量渲染、停止按钮发送取消、错误/重试沿用阶段① 行为；Bridge 新增四方法（preview 模式抛明确降级）。
- 阶段④ 补丁入口（v0.1.10 后随代码浏览器阶段④一同落地，PR 待开）：助手消息内识别 `\`\`\`diff` 围栏后露出「预览并应用到代码」按钮，调用 PatchPreviewDialog 三栏渲染；详见 `docs/features/code-browser/changelog.md` 阶段④条目。

## 2026-08-22（阶段① 对话基础实现）

- core `llm.rs`：新增 `ChatMessage` 与 `LlmClient::chat_messages`（多轮）；原 `chat(system, user)` 改为其封装，invoke 路径行为不变。新增 `CHAT_SYSTEM_PROMPT` 常量。
- 桌面壳新增 IPC `chat_completion`：复用 ConfigLayers 配置链，system 提示词由 Rust 侧前置（前端仅可传 user/assistant，注入 system 直接拒绝）；未配置/失败返回中文错误，不做降级 SOP。
- 前端：侧栏新增一级导航（能力工具箱 ⇄ AI 对话）；新增 `ChatView.vue`（多轮消息、生成中、停止=丢弃在途结果、失败重试、超长输入提示）；`Bridge` 新增 `chat()`（预览模式明确降级）。
- 模型输出按不可信 Markdown 渲染（ADR-002：覆写 html/link/image renderer，零新依赖），代码块带复制按钮，注入样本实测通过。
- 会话仅内存态；流式、请求级取消、本地会话管理分别属阶段②④，未实现。

## 2026-08-22

- 将 AI 对话登记为独立桌面 Feature，规划为四阶段：对话基础、会话管理、能力协作、流式与跨平台完善。
- 明确对话页不做隐式执行 Agent，能力调用必须经过用户确认。
