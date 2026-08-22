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

v0.1.0（2026-08-21 首个 GitHub Release，dmg + msi，tag `v0.1.0`）

## 进行中

- **桌面端能力导入/导出/删除**（`feature-2026-08-desktop-import-export`）：用户叠加层 `~/.elwright/`，桌面壳读写自定义能力，app 更新不丢。v1 范围：单项导出、可删除自定义项、浏览器预览导出走 Blob 下载。**实现与本地验证已完成**，待 CI + 用户实机确认后归档。CLI 同步获得 `ew delete` 与合并视图（`ls` SRC 列）。

## V1（短期，做完即发版）

1. **发 v0.1.1**：更新检查按钮（已合入 main 的 f344b69）进正式产物——版本号三处同步（`src-tauri/Cargo.toml` / `tauri.conf.json` / `src/package.json`）+ 打 tag，CI 自动出包。
2. **剩余资源导入**（公司机原版，纯标准库零依赖底线）：10 个脚本 entry + 4 个 knowledge doc，清单见 `capabilities.json` 中文件缺失项。
3. **api-doc-formatter 归类决策**（脚本 vs 技能，架构方案 §11 遗留）：定了改注册表 `type` 字段即可。
4. **script-tools Feature 文档补齐**：`docs/features/script-tools/` 缺 behavior / architecture / changelog。
5. 两个 bugfix 任务目录（ew-broken-pipe、missing-skill-sops）补 STATUS.md，与归档流程对齐。

用户动作（不占开发排期，阻塞对应任务归档）：

- 归档 10 个 ready-for-release 任务目录（active→archive，维护方案规定人执行）。
- 公司机实装一次 msi（stage4-release 验证收尾）。
- 用真实 LLM 端点 `ew invoke` 复验一次（stage2 验证收尾）。

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
