# ADR-004：对话长上下文——core 侧字符预算滑动窗口

日期：2026-08-31 · 状态：已接受（实施随 feature-2026-08-chat-long-context）
关联：ADR-003（流式与取消）之后阶段④余项；ROADMAP「AI 对话」收尾

## 背景

`chat_completion` 与 `chat_completion_stream` 均为 `system + 全量 history` 原样转发
（src-tauri/src/core/commands.rs:240/887）。会话一长（阶段②起本地会话永久保存）：
请求体超模型上下文上限 → API 报错；或不出错但成本随轮次线性膨胀。长会话是
已保存会话被复用的必然场景，需要明确策略。

## 决策

**core 侧本地字符预算滑动窗口**，两个 chat 命令共用一个收口函数：

1. **收口点在 core**（新模块 `core::chat_context::fit_messages`）：
   - 始终保留：system 提示 + 最新一条 user 消息；
   - 其余消息**从最新往最旧**按预算保留，放不下的最旧消息**整条丢弃**（不切半条，
     半条上下文易误导模型）；
   - 例外：最新 user 本身超预算时，截去其中段（保留头尾各半预算内），尾部标注
     `…（超长截断）`。
2. **预算**：默认常量 24000 字符（中文约 8-12k token 量级）；LLM 配置链新增可选
   flat 字段 `contextBudgetChars`（serde(default)，旧配置零迁移）；**profile 级
   覆盖后置**（见拒绝项）。
3. **静默裁剪**：不向对话流插入提示、不发新事件类型；预算值后续在模型设置页展示。
4. 单测覆盖：预算边界 / 最新 user 必留 / 超长单条中段截断 / 全部丢弃时不丢 system。

## 理由

- 主干红线「LLM 可选、离线基础可用」：截断是纯本地逻辑，零依赖、可预测、零延迟；
- 收口在 core 与既有安全模型一致（system 由 Rust 侧控制、CLI/桌面同链路）；
- 字符估算避开本地 tokenizer（引依赖、词典维护成本高，估算对本用途足够）。

## 拒绝 / 后置

- **LLM 摘要压缩旧消息**：依赖 LLM 在线、慢且贵、结果不稳定；作为可选增强后置，
  不做唯一路径。
- **profile 级预算覆盖**：ADR-001 档案机制可自然扩展，但首版不扩 serde 结构，避免
  与本批次解耦原则冲突；需要时另起小任务。
- **前端截断**：破坏「core 收口」一致性，CLI 未来复用对话链路时会漏。
- **UI 裁剪提示**：每轮打扰，静默 + 设置展示预算值足够。

## 跨平台完善（阶段④另一余项）

不进本 ADR 定案：以 `docs/work/active/PENDING-REAL-MACHINE-CHECKLIST.md` 与已知
WebView2 差异（字体回退/滚动条/个别 CSS/快捷键 Ctrl↔Cmd）为输入，逐项走独立
bugfix 任务目录，Windows 真机可用时集中点验。

## 验证计划

`core::chat_context` 单测 ≥5；`chat_completion` / `chat_completion_stream` 各加
mock runtime 用例验证裁剪生效；既有 78 核心 + 60 vitest 不回归；真机：长会话
（>预算）连续对话不报错、回复与最近上下文一致。
