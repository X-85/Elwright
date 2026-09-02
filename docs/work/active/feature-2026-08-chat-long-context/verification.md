# verification：AI 对话长上下文（ADR-004）

日期：2026-08-31

## 【自动化】

- `core::chat_context` 6 单测全绿：预算内原样返回 / 空历史 noop / 超预算最旧先丢
  且相对顺序保持 / 最新超长中段截断（头尾保留 + 标注）/ 极小预算仅留截断的最新条 /
  末尾非 user 消息的健壮性。
- 新集成测试 `tests/chat_completion_ipc.rs`：进程内自起 127.0.0.1 mock LLM 服务
  （记录请求体原文、返回 OpenAI 兼容响应），IPC 调 `chat_completion` 发送 ~9.6k 字符
  历史（预算 3000），断言真实请求体：system 前置、最新 user 完整无损、
  历史（不含 system）总字符 ≤ 预算、最旧轮次整条消失、较新轮次保留。
- 全量闸门：cargo test 84 核心（含 6 新）+ 集成 10 用例全绿；clippy --all-targets
  -D warnings 0 警告；cargo fmt 无差异；eslint 0 告警；vitest 60/60；vite build 成功。

## 【自动化】CLI 冒烟

- `ew config set context_budget_chars 12345` → `ew config` 显示
  `context : 12345 字符（对话上下文预算，ADR-004）`；config.json 经 UserConfigFile
  结构化落盘（flat + profiles + active_profile 共存）。
- 带 profiles 的配置执行 `set` 后档案与 active 完整保留（顺手修的数据丢失隐患回归）。

## 【手测】真机（待用户）

- 长会话（超预算）真机连续对话不报错、回复与最近上下文一致——需真实 LLM 端点，
  已留 PENDING-REAL-MACHINE-CHECKLIST 语境；桌面 UI 与流式链路零改动，风险低。
