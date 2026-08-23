# Plan：script-tools Feature 文档补齐 + 路线图终端条目同步

## 目标

1. `docs/features/script-tools/` 只有 README.md（内容还停留在「第一批 3 个通用脚本」），补齐 behavior / architecture / changelog，并改写 README 为当前真实状态（内置仅 `text-stats` 示例）。
2. 同步 `docs/ROADMAP.md`：集成终端 v1 已由用户真机验证并归档（`docs/work/archive/feature-2026-08-integrated-terminal/`），「进行中」条目应移除、里程碑补记；V1 增补「发 v0.1.2（含终端）」发版项。

## 范围

- 纯文档任务，不改代码、不改注册表。
- 事实来源：`capabilities.json`（当前 3 条示例）、`resources/tools/text-stats/text_stats.py`、`src-tauri/src/core/executor.rs`、`src-tauri/src/bin/ew.rs`（run 命令校验）。

## 非目标

- 不恢复已移除的 doc-keyword-search / xlsx-to-md / docx-to-md 文档内容（历史见 git 与 archive）。
- 不更新 AGENTS.md「当前进度」以外的章节。

## 实现步骤

1. 改写 `docs/features/script-tools/README.md`（面向 text-stats + 用户叠加层约定）。
2. 新增 behavior.md / architecture.md / changelog.md，风格对齐 `docs/features/llm-invoke/`。
3. 更新 ROADMAP（进行中移除终端条目、V1 增补发版项并勾掉本文档任务、已完成里程碑补记）。
4. AGENTS.md「当前进度」一句话同步。

## 风险与验证方式

- 风险：文档与代码行为不一致。验证：逐条对照 executor.rs / ew.rs / capabilities.json 的实际逻辑（解释器映射、类型校验、报错文案）。
- ROADMAP 属活文档，本次改动即最终状态，无需另行验证。
