# Plan：AI 对话 阶段②（会话管理）

## 目标

桌面壳 AI 对话页增加本地会话存储与生命周期：新建/切换/重命名/删除会话，消息自动持久化。刷新或重开 app 后能恢复上次的对话。API Key 永不进入会话文件（messages 只有 role/content）。

对齐 `docs/features/chat/README.md` 阶段 2 与 architecture.md「会话存储属于桌面壳」。

## 技术方案

### 存储（`src/chat_store.rs`，桌面壳模块，不进 core）

- 路径：`~/.elwright/chats/<id>.json`（复用 `registry::user_root()`，子目录 `chats/`；`ELWRIGHT_USER_ROOT` 覆盖同生效，便于测试）。
- `ChatSession { id, title, created_at, updated_at, messages: Vec<llm::ChatMessage> }`；复用 core 的 ChatMessage（role/content），不引入新消息类型。
- 摘要 `ChatSessionSummary { id, title, updated_at }`（list 不读全量 messages，只读元字段）。
- 函数：`list_sessions`（按 updated_at 倒序）、`load_session`、`save_session`（upsert，写时更新 updated_at）、`delete_session`、`new_session_id`（时间戳+短随机，避免依赖 uuid crate）。

### IPC（`main.rs`）

- `chat_list_sessions -> Vec<ChatSessionSummary>`
- `chat_load_session(id) -> Option<ChatSession>`
- `chat_save_session(id, title, messages) -> Result`（id/title/messages 入参，updated_at 服务端写）
- `chat_delete_session(id) -> Result`
- 重命名走 save（前端带新 title 调 save）。

### 前端（`bridge.ts` + `ChatView.vue`）

- bridge：`ChatSessionSummary`/`ChatSession` 类型 + `listChatSessions/loadChatSession/saveChatSession/deleteChatSession`；预览模式抛「【预览模式】」（浏览器无文件系统）。
- ChatView：左列会话列表（新建/选择/重命名/删除）+ 右侧消息区。挂载时 load 列表 + 最近会话；每条消息后自动 save（title 取首条 user 消息前 24 字，无则「新对话」）；切换/删除后状态正确。

## 非目标

- 流式/请求级取消（阶段④）；能力协作（阶段③）；多设备同步；会话加密。
- 会话内搜索、置顶、导出（远期）。

## 风险与验证

- 并发写：单用户本地，无锁需求；save 全量覆写单文件。
- 文件损坏：load 反序列化失败时跳过该会话（不崩列表）。
- chat_store 单测：用 `ELWRIGHT_USER_ROOT` 指向 temp 目录测 list/save/load/delete/rename/upsert + 损坏文件跳过。
- 真机：刷新 app 后会话恢复；切换/删除/重命名即时反映。
