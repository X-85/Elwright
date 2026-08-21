# 技能型能力的 degradeDoc 文件缺失

## 问题

`capabilities.json` 中 6 个技能型能力的 `degradeDoc` 指向的 SOP 文件均不存在（阶段 0 登记时只规划了路径）。离线降级时 `ew invoke` 只能打印「SOP 文件不存在」，降级体验名存实亡。

## 原因

阶段 0 只登记引用、不导入文件（架构方案 §8「本期不移动/改写任何现有 skill 文件」）；阶段 2 实现降级链路时仅补了 `tech-grill-sop.md` 作种子验证，其余 5 个遗留。

## 修改范围

新增 5 个文件（纯资源文档，无代码改动）：

- `resources/docs/prompt-optimizer-sop.md` — 提示词优化六要素模板
- `resources/docs/knowledge-sharing-sop.md` — 会话转分享文档的结构与脱敏流程
- `resources/docs/work-summary-sop.md` — 周报/月报五段模板与量化规则
- `resources/docs/api-doc-formatter-sop.md` — 接口文档标准格式与整理流程
- `resources/docs/skill-selpoint-sop.md` — 需求评审三维打分 + MoSCoW 归类法

SOP 内容为「该技能由 LLM 执行时所用方法的离线手动版」，与 tech-grill 同一写法：用途 / 模板 / 使用方法 / 自查。

## 风险与影响

- 注册表未动（条目与 degradeDoc 路径原本就正确），行为无变化，只是降级文案从「文件不存在」变为完整 SOP。
- 与阶段 3b（Codex 进行中）无文件交集：只新增 resources/docs/ 与本任务目录。
- 遗留小项：`docs/features/llm-invoke/changelog.md` 可补一行「SOP 全部到位」，因该文件可能被 3b 同时编辑，留待 3b 合入后再补，避免同文件冲突。
