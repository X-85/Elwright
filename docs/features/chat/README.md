# AI 对话

## 定位

AI 对话是 Elwright 桌面应用中的自然语言工作入口。用户可以和已配置的大模型进行多轮交流，也可以在明确确认后关联并调用 Elwright 的脚本、知识和技能能力。

它不是脱离 Elwright 能力体系的普通聊天工具，也不是默认自动执行任务的 Agent。LLM 负责理解、生成和辅助编排，实际能力执行仍由现有共享核心完成。

## 当前状态

阶段①（对话基础）已实现（2026-08-22）；阶段②（会话管理）代码完成（2026-08-23）：会话侧栏（新建/切换/重命名/删除）、每会话一个 JSON 文件存 `~/.elwright/chats/`、自动保存。非流式；待真机 GUI 验证。阶段③④未实现。

## 分阶段范围

1. **对话基础**：独立 AI 对话页面、多轮文本消息、Markdown/代码块、发送/停止/重试、模型状态展示。✅ 已实现（2026-08-22）。
2. **会话管理**：本地新建/切换/重命名/删除会话，默认本地保存，敏感内容最小化持久化。✅ 代码完成（2026-08-23，待真机验证）。
3. **能力协作**：从对话中手动关联能力；模型可以推荐能力和草拟参数，但执行前显示目标与影响并由用户确认。
4. **流式与完善**：流式输出、取消请求、长上下文处理、macOS/Windows 真机验证和性能优化。

## 设计原则

- LLM 是增强，不是地基；未配置或不可达时显示明确状态，已有脚本、知识、终端和技能 SOP 仍可用。
- 对话默认只发送用户明确输入的内容，不自动读取屏幕、剪贴板、麦克风或摄像头。
- API Key 不返回前端、不写入会话；会话默认只保存在本地用户目录。
- 模型不能隐式执行脚本、写文件或访问外部系统；高影响操作必须用户确认。

## 相关基础

- LLM 客户端与配置：`src-tauri/src/core/llm.rs`（含 `ChatMessage`/`chat_messages` 多轮接口与 `CHAT_SYSTEM_PROMPT`）
- 桌面对话 IPC：`src-tauri/src/main.rs` `chat_completion` + 会话四命令（`chat_list_sessions` / `chat_load_session` / `chat_save_session` / `chat_delete_session`，system 提示词 Rust 侧前置）
- 会话存储（桌面壳模块，非 core）：`src-tauri/src/chat_store.rs`（`~/.elwright/chats/<id>.json`，一文件一会话）
- 桌面 Bridge：`src/lib/bridge.ts`（`chat()` 与会话四方法）；不可信渲染：`src/lib/safeMarkdown.ts`（ADR-002）
- 对话页面：`src/components/ChatView.vue`；模型设置：`src/components/LlmSettings.vue`
- 技能调用与降级：`src-tauri/src/core/invoke.rs`
- 路线图：`docs/ROADMAP.md`
