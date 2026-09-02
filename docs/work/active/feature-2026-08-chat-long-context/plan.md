# plan：AI 对话阶段④余项——长上下文（ADR-004）+ 跨平台完善

日期：2026-08-31 · 来源：用户「直接执行 V2 剩余主干」定向，按 ROADMAP 顺序取 AI 对话余项

## 范围

1. **长上下文**（本任务主体）：按 [ADR-004](../../features/chat/decisions/ADR-004-long-context.md)
   实现 core 侧字符预算滑动窗口。
2. **跨平台完善**：不进本任务定案——以 PENDING-REAL-MACHINE-CHECKLIST 与 WebView2
   差异清单为输入，逐项独立 bugfix 任务目录（见 ADR-004 末节）。

## checklist

- [x] `core::chat_context::fit_messages` 新模块 + 单测 6（预算边界不裁 / 空历史 /
      最旧先丢且顺序保持 / 最新超长中段截断 / 极小预算 / 末尾非 user 健壮性）
- [x] `chat_completion` 与 `chat_completion_stream` 两处接入（共用 `assemble_chat_messages`）
- [x] LLM 配置链可选字段 `context_budget_chars`（serde default，None=24000 常量）+
      环境变量 `ELWRIGHT_LLM_CONTEXT_BUDGET_CHARS` + `ew config` 显示、`set` 可写
- [x] IPC 用例 `tests/chat_completion_ipc.rs`：自起 mock LLM 服务捕获真实请求体，
      断言 system 前置 / 最新 user 完整 / 历史收敛 ≤ 预算 / 最旧轮次被裁 / 较新轮次保留
- [x] 文档回填：chat behavior（长上下文节）/ architecture（阶段④余项节）/ changelog
      + ROADMAP「AI 对话」条目与里程碑
- [x] 闸门：cargo fmt/clippy(-D warnings)/test（84 核心 + 集成全绿）+ eslint/vitest 60/build

## 范围外顺手修（同文件数据丢失隐患）

- `ew config set` 原按 `BTreeMap<String,String>` 回写 config.json：配置含
  profiles/activeProfile（Q19 档案）时解析失败→静默清空整个文件。改为经
  `UserConfigFile` 读写（新增 `set_flat_field`），档案完整保留；CLI 冒烟验证
  「带档案 set 预算后 profiles/active 不丢」。

## 状态

已实现并验证（2026-08-31），随本提交进 main；任务目录归档待用户确认上线。
【手测】长会话（>预算）真机连续对话不报错——留 PENDING 清单（需真实 LLM 端点）。
