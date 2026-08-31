# ADR-003：AI 对话流式输出与请求级取消

## 状态

已接受（2026-08-31 立项；2026-08-31 随阶段④ 实现落地）

## 背景

当前 `chat_completion` 为一次性 IPC：`spawn_blocking` 里用 **reqwest blocking**
客户端（ADR-001，llm-invoke）等完整响应，结束后整段返回。阶段①的「停止」
只是前端丢弃在途结果（序号比对），后端请求照常完成——不省 token、不释放连接，
用户也要等全文才能看到第一个字。

阶段④ 目标：流式输出（逐 token 呈现）与请求级取消（真正中断后端读取）。

## 决策

1. **保留 blocking reqwest，不迁移 async。** 在 `spawn_blocking` 内以
   blocking Response 的 `Read` 流式逐块读取 OpenAI 兼容 `/v1/chat/completions`
   的 SSE 响应（`data: {...}` 行解析为手写极简实现，约 30 行，不引
   eventsource/async-openai 依赖——零依赖底线与架构方案 §9.2 锁定不变）。
2. **增量经 Tauri Channel 推送**，复用集成终端已验证的 Channel 模式
   （`__CHANNEL__:<id>` 接线，tier2 mock-runtime IPC 测试有现成覆盖方式）。
   消息为 JSON 事件：`{"type":"delta","text":…}` / `{"type":"done"}` /
   `{"type":"error","message":…}` / `{"type":"cancelled"}`。
3. **取消 = 显式命令 + 取消表**：新增 `chat_cancel(request_id)`；后端维护
   取消表（`Mutex<HashSet<u64>>`），读取循环每收到一块检查一次，命中即中断
   读取（drop Response 关闭连接）、发 `cancelled` 事件并从表中移除。
   前端「停止」改为发取消命令；序号比对作为第二道防线保留。
4. **前端增量渲染**：追加 delta 到当前 assistant 消息并节流重渲染
   （safeMarkdown/ADR-002 管线不变）；`done` 后走既有自动保存；`cancelled`
   保留已收到的部分文本并标注「（已停止）」。
5. **旧 `chat_completion` 保留但对话页切换到 `chat_completion_stream`**，
   给一个版本的过渡期后移除。长上下文策略（历史截断/摘要）不在本 ADR 范围。

## 落选方案

- **全面迁移 async（tokio reqwest + async SSE）**：技术最优，但违反 llm-invoke
  ADR-001 的 blocking 锁定，且形成 blocking/async 双客户端长期维护成本；
  CLI `ew invoke` 与能力执行链仍需 blocking。待未来出现强制因素再重议。
- **只做取消不做流式**：实现更小，但用户等待首字的核心体验问题不解决，
  阶段④要二次动同一片代码，不划算。
- **引 eventsource-parser 等 SSE 库**：OpenAI SSE 格式固定且简单，自研解析
  足够；引依赖违背零依赖底线。

## 验证

SSE 行解析单测 4/4（data 行 / [DONE] / 注释与非 JSON 容错 / 非 data 行）；
chat_completion_stream / chat_cancel 全量闸门通过；真机流式体验点验留档
PENDING-REAL-MACHINE-CHECKLIST。实现中发现并修复：code_browser 测试临时目录
并行撞名（时钟微秒精度），改原子序数唯一化。

## 后果

- 正面：首字延迟从「整段生成完成」降到「首个 chunk」；停止真正生效（省
  token、释放连接）；Channel 模式复用终端既有测试与实现经验。
- 代价：`spawn_blocking` 内长驻读取占用一个 blocking 线程直至完成/取消
  （池默认 512，可接受）；读取无逐块超时，依赖 reqwest 既有整体超时配置；
  IPC 测试需补 Channel 事件断言（mock runtime 已支持）。
- 风险：部分供应商 SSE 兼容性差异（如注释行、多余空行）——解析器按
  「跳过非 `data:` 行」容错；解析失败整段回退为非流式一次性返回。
- 后续：长上下文（历史截断/摘要）另立 ADR；tauri updater 等远期项不相关。
