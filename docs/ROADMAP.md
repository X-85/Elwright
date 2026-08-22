# Elwright 路线图（活文档）

> 本文是唯一的「接下来做什么」视图：要做什么、排在哪、状态如何，只看这里。
> `Elwright架构方案.md` §10/§11 是立项时的规划快照（记录决策与理由），不再随进度更新；
> `AGENTS.md` 的「当前进度」只保留一句话指针指向本文。
>
> 维护规则（配合 `resources/docs/AI_CODE_AGENT_MAINTENANCE.md`）：
> 立项 → 在 `docs/work/active/` 建任务目录，并在本文「进行中」登记一行；
> 上线确认 → 人执行归档（active→archive），条目从「进行中」移除，里程碑入「已完成」。
> 新想法不写代码时，直接在对应版块（V1/V2/远期）记一行，注明来源日期。

## 当前版本

v0.1.1（2026-08-22，tag `v0.1.1`，GitHub Release 附 dmg + msi。自 v0.1.0 新增：桌面导入/导出/删除、模型设置、检查更新按钮、注册表精简为 3 条示例、CLI delete/import 用户层）

## 进行中

（无——2026-08-22 全量归档 15 个任务目录至 archive/，回到干净基线，等待新功能立项）

## V1（短期，做完即发版）

1. ~~发 v0.1.1~~ **已完成（2026-08-22）**：更新检查按钮 + 桌面导入/导出/删除 + 模型设置 + 示例注册表全部进包，Release 流水线一次通过。
2. **script-tools Feature 文档补齐**：`docs/features/script-tools/` 缺 behavior / architecture / changelog。**2026-08-22 范围变化**：原 3 个脚本（doc-keyword-search/xlsx-to-md/docx-to-md）已随注册表精简移除，该 feature 目录需改写为面向 `text-stats` 示例或降级说明。
3. 两个 bugfix 任务目录（ew-broken-pipe、missing-skill-sops）补 STATUS.md ~~——已完成（归档时补建，已入 archive）~~。

~~剩余资源导入（公司机原版 10 脚本 + 4 知识文档）~~ **2026-08-22 作废**：内置注册表改为纯示例（3 条），个人能力不再进仓库，全部走用户叠加层导入。需要时可从公司机原版用 `ew export`/`ew import` 迁移。

用户动作（不占开发排期）：

- 公司机实装一次 msi（从 GitHub Release v0.1.0 或更新版本下载）。
- 桌面 app 配置真实 LLM 端点（⚙ 模型设置 → 测试连接 → 保存）并 invoke 一次，确认真实链路。
- 桌面 app 实机走一遍导入 → 徽标 → 删除循环。

## V2（中期）

- **批量导出**：多能力打一个包，`elwright-skill` schema 升级（数组形态或 manifest）。
- **二进制资源打包**：导出格式支持二进制（zip/base64）——jar-decompile 等含二进制工具链的能力需要。
- **隐藏内置能力**：注册表加 flag 或用户配置层记录隐藏项，列表可精简。
- **注册表目录自动扫描**：架构方案 §9.2 明确留作后续增强，静态 JSON 仍是 v1 底线。
- **Linux 支持**：架构方案 §11 列为后续。

## 远期

- **完整自动更新**：tauri updater + minisign 签名，静默升级（当前是检查按钮 + 跳下载页）。
- **能力市场 / 远端 registry**：填 URL 拉取能力清单与脚本，社区分享去中心化 GitHub 之外的路径。

## 已完成里程碑

- 2026-08-21 阶段 1：Rust 核心 + CLI `ew`（ls/run/view/invoke），公司机跑通。
- 2026-08-21 阶段 2：LLM 接入（reqwest blocking，ADR-001）+ 离线降级 SOP；6 个技能型 SOP 补齐。
- 2026-08-21 阶段 3/3b：Vue 3 前端 + Tauri 壳，四 IPC 命令，浏览器/桌面双适配 bridge。
- 2026-08-21 阶段 4：CI 六 job 三平台、release.yml tag 发版、dmg + msi，v0.1.0 上 GitHub Release。
- 2026-08-22 增强：`ew config` 四档 LLM 配置链、能力导入/导出（CLI）、更新检查按钮、3 个通用脚本（doc-keyword-search / xlsx-to-md / docx-to-md）、msi CI 打包（WiX 重名基文件修复）。
- 2026-08-22 注册表精简：内置注册表从 24 条种子清单改为 3 条真实示例（text-stats / capability-types / weekly-report），个人能力全面转向用户叠加层 `~/.elwright/`；旧脚本/SOP/知识文档移除，「公司机原版批量导入」计划作废。
- 2026-08-22 桌面端模型设置：⚙ 弹层（key 打码/测试连接/来源标签），写用户层与 `ew config` 互通。
- 2026-08-22 全量归档：15 个任务目录（阶段 1–5 全程 + CI/发布/导入导出/模型设置/注册表精简）active→archive，回到干净基线。
