# AI 对话阶段④计划：流式输出与请求级取消（ADR-003）

## 范围

- llm.rs：chat_messages_streaming（blocking Read 逐块读 SSE，手写 data: 行解析，
  每块检查取消表；供应商不兼容时由调用方回退非流式）。
- 命令：chat_completion_stream（Channel 推 delta/done/error/cancelled JSON 事件）
  + chat_cancel（取消表）；chat_completion 保留过渡。
- 前端：bridge.chatCompletionStream（Tauri Channel）/chatCancel；ChatView
  桌面走流式增量渲染（50ms 节流）、停止真取消、cancelled 保留部分文本标注；
  浏览器预览维持旧降级路径。

## 非目标

- 长上下文策略（历史截断/摘要，另立 ADR）。
- 全面 async 迁移（ADR-003 落选方案）。
