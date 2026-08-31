# plan：AI 对话阶段④余项——长上下文（ADR-004）+ 跨平台完善

日期：2026-08-31 · 来源：用户「直接执行 V2 剩余主干」定向，按 ROADMAP 顺序取 AI 对话余项

## 范围

1. **长上下文**（本任务主体）：按 [ADR-004](../../features/chat/decisions/ADR-004-long-context.md)
   实现 core 侧字符预算滑动窗口。
2. **跨平台完善**：不进本任务定案——以 PENDING-REAL-MACHINE-CHECKLIST 与 WebView2
   差异清单为输入，逐项独立 bugfix 任务目录（见 ADR-004 末节）。

## checklist（实施时勾选）

- [ ] `core::chat_context::fit_messages` 新模块 + 单测 ≥5（预算边界 / 最新 user 必留 /
      超长单条中段截断 / system 必留 / 空消息列表）
- [ ] `chat_completion` 与 `chat_completion_stream` 两处接入（共用 fit）
- [ ] LLM 配置链可选 flat 字段 `contextBudgetChars`（serde default 24000）+ `ew config` 可见
- [ ] IPC mock runtime 用例：长会话经裁剪后请求体收敛（≤预算 + system + 最新 user）
- [ ] 文档回填：chat behavior/architecture/changelog + ROADMAP「AI 对话」条目
- [ ] 闸门：cargo fmt/clippy/test 全绿 + eslint/vitest/build 全绿

## 状态

ADR 已写入 features/chat/decisions/ADR-004；实施待用户对 ADR 无异议后开工
（沿用 Q18/Q19/Q20「先 ADR 再实施」两阶段协议；本次 ADR 不单独开 PR，随本目录入库）。
