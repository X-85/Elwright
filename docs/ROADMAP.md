# Elwright 路线图（活文档）

> 本文是唯一的「接下来做什么」视图：要做什么、排在哪、状态如何，只看这里。
> `Elwright架构方案.md` §10/§11 是立项时的规划快照（记录决策与理由），不再随进度更新；
> `AGENTS.md` 的「当前进度」只保留一句话指针指向本文。
>
> 维护规则（配合 `resources/docs/AI_CODE_AGENT_MAINTENANCE.md`）：
> 立项 → 在 `docs/work/active/` 建任务目录，并在本文「进行中」登记一行；
> 上线确认 → 人执行归档（active→archive），条目从「进行中」移除，里程碑入「已完成」。
> 新想法不写代码时，直接在对应版块（V1/V2/远期）记一行，注明来源日期。

## 主干红线与准入

Elwright 主干只做能够强化以下闭环的功能：**沉淀能力 → 调用能力 → 记录工作上下文 → AI 辅助理解与组织 → 用户确认执行 → 产出可复用结果**。

- 必须保持：LLM 可选、离线基础可用、用户明确授权、数据本地优先、CLI/桌面共享能力核心。
- 不进入主干：全自动 Agent、隐式命令/文件执行、默认采集屏幕/终端/剪贴板等数据、娱乐化桌宠、通用效率软件替代、完整 SSH/服务器管理客户端。
- 不能直接服务闭环的个人需求，可在 `codex/experiment-*` 或其他独立实验分支验证；实验代码和依赖不进入主干，除非另行立项并证明具备清晰、通用的工作流价值。

## 当前版本

v0.1.5（2026-08-24，tag `v0.1.5`。基于 v0.1.4 新增：终端两 bug 修复（无回显 / 第二 tab 不能输入）+ 终端八项交互优化（ZCode 风格）+ 设置中心第一阶段（三态主题）+ AI 会话图标与新建按钮图标化 + 工程质量治理第一档（CI clippy+fmt 闸门 + 前端 vitest））

## 开发预览环境约束（本机 Windows / GNU-only）

> 适用：公司机（Win11，原仅 GNU+MinGW 工具链，无 MSVC 链接器）。家里 macOS 不受影响。
> 根因：`src/lib/bridge.ts` 有 `browserBridge` / `tauriBridge` 双实现，`npm run dev`（vite，浏览器）走 `browserBridge`。

- **`npm run dev` 只覆盖「查看类」功能**：真正可用仅 `listCapabilities`（`/api/capabilities`）、`viewDoc`（`/api/file`）、`exportCapability`（Blob 下载）、`checkUpdate`（直连 GitHub）。其余在浏览器里是 stub / 抛错：**runScript / invokeSkill（永远降级，不真调 LLM）/ importCapability / deleteCapability / getLlmConfig / setLlmConfig / chat / 会话持久化 / openTerminal** 全部失效或抛「【预览模式】…请用桌面应用」。
- **功能开发无法在本机 `npm run dev` 验证**：终端、AI 对话、能力增删、模型配置、技能调用等核心功能，浏览器预览验不了，必须进真机运行时。
- **MSVC 决策（2026-08-24，终：CI-only 不装）**：曾尝试本机装 VS Build Tools（VCTools + Win11 SDK）以本地 `tauri dev`/`tauri build` 点测功能，但公司网关封锁 `download.visualstudio.microsoft.com`（403 类别封锁，pi 代理亦绕不过；本机也无软件中心 / 内部镜像）。**最终决定维持 CI-only**：本机不装 MSVC，桌面 msi 继续由 GitHub `release.yml` 出；本地开发用 GNU 编 `ew` + 浏览器 `npm run dev` 预览 UI（仅查看类功能可用，见上「npm run dev 只覆盖查看类功能」）。GNU 仍是本机唯一工具链。
- **漂移风险**：浏览器预览依赖 `vite` 的 `/api/*` 开发中间件镜像 Rust 命令；改注册表 schema 时这套中间件与 Rust 侧需同步。
- **渲染差异**：WebView2（真机）≠ Chrome（`vite dev`）：排版基本一致，但字体回退 / 滚动条 / 个别 CSS / Tauri CSP 以真机为准。

## 进行中

- **设置中心第一阶段**（`feature-2026-08-settings-center-phase1`）：统一设置入口，交付常规/外观/模型设置分组与系统、浅色、深色主题偏好；终端主题同步和更多常规配置留后续阶段。
- **人与人消息会话第一阶段**（`feature-2026-08-messaging-phase1`）：消息会话客户端基础，支持本地文字/图片/表情和会话状态；跨设备消息传输与实时协作空间后置。

## V1（短期，做完即发版）

1. ~~发 v0.1.1~~ **已完成（2026-08-22）**：更新检查按钮 + 桌面导入/导出/删除 + 模型设置 + 示例注册表全部进包，Release 流水线一次通过。
2. ~~script-tools Feature 文档补齐~~ **已完成（2026-08-22）**：目录改写为面向 `text-stats` 示例 + executor 通用执行行为，README/behavior/architecture/changelog 四件套齐。
3. 两个 bugfix 任务目录（ew-broken-pipe、missing-skill-sops）补 STATUS.md ~~——已完成（归档时补建，已入 archive）~~。
4. ~~发 v0.1.2 + v0.1.3 hotfix~~ **已完成（2026-08-23）**：v0.1.2 集成桌面壳内集成终端（PTY + xterm.js）；v0.1.3 hotfix 修「检查更新」按钮 IPC 序列化 bug（`UpdateInfo` 字段命名），见 `docs/work/archive/bugfix-2026-08-check-update-no-prompt/`。
5. ~~发 v0.1.4（含 AI 对话 阶段①②）~~ **已完成（2026-08-23）**：`feat/chat` 合并 → `main`（commit `20d4199`）；首次 `release.yml` run #5 因 dmg upload 与并行 CI 的 macos job 冲突失败，删 tag 重打后 run #6 全绿，dmg+msi 已上 GitHub Release。`bugfix-2026-08-check-update-no-prompt` 与 `enhancement-2026-08-script-tools-docs` 一并归档到 `docs/work/archive/`。
6. **发 v0.1.5（终端 bugfix + 体验批次 + 工程质量第一档）**：`bugfix/2026-08-app-shell-feedback` 合并 → `main`（本条为发版清单项）。

~~剩余资源导入（公司机原版 10 脚本 + 4 知识文档）~~ **2026-08-22 作废**：内置注册表改为纯示例（3 条），个人能力不再进仓库，全部走用户叠加层导入。需要时可从公司机原版用 `ew export`/`ew import` 迁移。

用户动作（不占开发排期）：

- 公司机实装一次 msi（从 GitHub Release v0.1.0 或更新版本下载）。
- 桌面 app 配置真实 LLM 端点（⚙ 模型设置 → 测试连接 → 保存）并 invoke 一次，确认真实链路。
- 桌面 app 实机走一遍导入 → 徽标 → 删除循环。

## V2（中期）

### 主干优先

- **AI 对话**：分四阶段实现——①多轮对话页面与基础状态；②本地会话管理；③用户确认式能力协作；④流式输出、取消、长上下文和跨平台完善。模型不配置或不可达时保持明确降级提示，能力执行继续复用现有 core。
- **人与人消息会话**：分三阶段实现——①客户端本地消息会话；②轻量身份/邀请与一对一消息传输；③从消息会话升级实时协作空间。第一阶段不建设通用社交能力。
- **设置中心后续阶段**：终端主题、字体和字号同步；常规中的启动视图、更新策略与语言；模型档案等配置，均以本地优先、少而明确为准入标准。
- **工作工具栏（Workbench）**：先实现能承接能力调用的收藏/最近使用、Todo、书签和少量高频研发转换工具；今日记录、更多通用转换工具和桌宠联动后置。内置工具优先本地运行，复杂扩展继续使用能力注册表。
- **工程质量治理**（2026-08-23 立项，起因：终端两 bug 均落在保障空白区——IPC 接缝与前端组件，见 `docs/work/archive/bugfix-2026-08-app-shell-feedback/`）。三档：
  - **第一档（立即）**：CI 加 clippy + rustfmt 检查；前端引入 vitest，先覆盖纯逻辑模块（safeMarkdown / theme / bridge 纯函数）。
  - **第二档（已完成 2026-08-24，`enhancement-2026-08-quality-tier2-e2e`）**：分层 e2e 冒烟——IPC 层用 tauri mock runtime 真协议测试（`src-tauri/tests/terminal_ipc.rs`，macOS/Linux 走真 PTY 全链路，Windows+CI 跳真 PTY 用例），浏览器层用 Playwright chromium（`src/e2e/`，接缝 + 降级守卫）；验证清单 `【自动化】/【手测】` 标记约定已写入 AI_CODE_AGENT_MAINTENANCE.md §6。tauri-driver 弃选（Windows CI 无 WinAppDriver），取舍见 `docs/features/engineering-quality/decisions/ADR-001-e2e-layering.md`。
  - **第三档（按需）**：eslint（风格已统一，低优先）；覆盖率门槛（等测试有存量）。

### 后置验证

- **脑图 MVP**：本地脑图、节点编辑以及与 Todo/能力的关联。AI 草稿、复杂知识整理和桌宠入口须在主干闭环稳定后验证，不做多人协作、云同步或通用白板。
- **工程图 MVP**：流程图可视化编辑、Mermaid 生成/导出、与 Todo/能力的关联，优先覆盖部署和故障排查流程。时序图、复杂源码双向同步、AI 日志分析和终端/服务器关联须在 MVP 被验证有通用价值后另行排期。

### 暂不进入主干

- **桌宠**：只在独立实验分支验证托盘、快捷入口和任务状态反馈，不进入主干近期排期。
- **扩展工作台工具**：今日记录、泛化转换工具、复杂笔记和效率组件先不排主干。
- **批量导出**：多能力打一个包，`elwright-skill` schema 升级（数组形态或 manifest）。
- **二进制资源打包**：导出格式支持二进制（zip/base64）——jar-decompile 等含二进制工具链的能力需要。
- **隐藏内置能力**：注册表加 flag 或用户配置层记录隐藏项，列表可精简。
- **注册表目录自动扫描**：架构方案 §9.2 明确留作后续增强，静态 JSON 仍是 v1 底线。
- **Linux 支持**：架构方案 §11 列为后续。

## 远期

- **完整自动更新**：tauri updater + minisign 签名，静默升级（当前是检查按钮 + 跳下载页）。
- **能力市场 / 远端 registry**：填 URL 拉取能力清单与脚本，社区分享去中心化 GitHub 之外的路径。
- **桌宠（实验方向）**：仅作为能力与任务的低打扰快捷入口；不作为主干近期功能，不做游戏化或自动化 Agent。先在独立实验分支验证真实工作流价值。
- **扩展工作台工具（实验方向）**：通用 Todo/笔记增强、更多转换工具、时序图/架构图/ER 图、SSH/服务器管理等仅限独立实验分支；不以“替代现有成熟软件”为目标，满足主干红线后才可另行立项。

## 已完成里程碑

- 2026-08-23 应用壳布局第一阶段：顶部全局操作栏、能力工具箱/AI 对话图标化、左右面板显隐、按需展开的终端抽屉。任务已归档至 `docs/work/archive/enhancement-2026-08-app-shell-layout/`。

- 2026-08-21 阶段 1：Rust 核心 + CLI `ew`（ls/run/view/invoke），公司机跑通。
- 2026-08-21 阶段 2：LLM 接入（reqwest blocking，ADR-001）+ 离线降级 SOP；6 个技能型 SOP 补齐。
- 2026-08-21 阶段 3/3b：Vue 3 前端 + Tauri 壳，四 IPC 命令，浏览器/桌面双适配 bridge。
- 2026-08-21 阶段 4：CI 六 job 三平台、release.yml tag 发版、dmg + msi，v0.1.0 上 GitHub Release。
- 2026-08-22 增强：`ew config` 四档 LLM 配置链、能力导入/导出（CLI）、更新检查按钮、3 个通用脚本（doc-keyword-search / xlsx-to-md / docx-to-md）、msi CI 打包（WiX 重名基文件修复）。
- 2026-08-22 注册表精简：内置注册表从 24 条种子清单改为 3 条真实示例（text-stats / capability-types / weekly-report），个人能力全面转向用户叠加层 `~/.elwright/`；旧脚本/SOP/知识文档移除，「公司机原版批量导入」计划作废。
- 2026-08-22 桌面端模型设置：⚙ 弹层（key 打码/测试连接/来源标签），写用户层与 `ew config` 互通。
- 2026-08-22 全量归档：15 个任务目录（阶段 1–5 全程 + CI/发布/导入导出/模型设置/注册表精简）active→archive，回到干净基线。
- 2026-08-22 集成终端 v1：底部抽屉 + 多 tab + 本地 shell（portable-pty + Tauri Channel）+「在终端中运行」联动，xterm.js (WebGL) 渲染；代码完成并由用户真机端到端验证通过，任务目录已归档（`docs/work/archive/feature-2026-08-integrated-terminal/`）。已随 v0.1.2 / v0.1.4 发版。
- 2026-08-23 AI 对话阶段①②：阶段① 独立对话页 + 多轮消息 + 安全 Markdown 渲染（ADR-002）+ 发送/停止/重试 + 模型状态展示；阶段② 会话侧栏 + `~/.elwright/chats/` 一文件一会话 + 自动保存 + 错误消息不落盘，零新依赖（手写 ISO8601 + AtomicU64 id）。真机验证后顺手修了 4 处 UI 反馈（保存表单自动关闭 / 复制按钮深色背景可见 / 重命名保护不被自动覆盖 / 引导诊断日志）；合并提交 `282d081` `6a8e83c`。三任务目录归档：phase1 / phase2 / bugfix-real-machine-issues。
- 2026-08-23 v0.1.2 + v0.1.3 hotfix：v0.1.2 集成终端 v1 入包；v0.1.3 hotfix 修「检查更新」按钮 IPC 序列化 bug（`UpdateInfo` 字段命名）。见 `docs/work/archive/bugfix-2026-08-check-update-no-prompt/`。
- 2026-08-23 v0.1.4：集成终端 v1 + AI 对话阶段①② + 一键安装脚本（macOS/Windows）进包。`feat/chat` 合并 commit `20d4199`；首次 `release.yml` run #5 dmg upload step 跟并行 CI 的 macos job 冲突失败，删 tag 重打后 run #6 全绿，dmg+msi 已上 Release。归档目录：chat phase1 / phase2 / bugfix-real-machine-issues / check-update-no-prompt / script-tools-docs。
- 2026-08-24 工程质量治理第二档（分支 `enhancement/2026-08-quality-tier2-e2e`）：分层 e2e——IPC 层 tauri mock runtime 真协议测试（6 用例，macOS/Linux 真 PTY 全链路）+ 浏览器层 Playwright chromium（5 用例，接缝与降级守卫）；配套 IPC 命令层从 main.rs 下沉 `core/commands.rs`（AppCtx 注入，行为零变化）+ 存量测试环境变量锁统一（并行缺陷修复）。验证清单 `【自动化】/【手测】` 标记约定入 AI_CODE_AGENT_MAINTENANCE.md §6；新 Feature 文档 `docs/features/engineering-quality/`（含 ADR-001 e2e 分层取舍）。tauri-driver 弃选（Windows CI 无 WinAppDriver）。
- 2026-08-24 应用壳反馈 bugfix + 终端体验批次（随 v0.1.5 进包）：修两个真机 bug（终端无回显——Tauri Channel 用法写反；第二 tab 不能输入——组件复用换 prop 不重接线，结构重写为 per-tab 实例）+ 八项交互优化（顶栏直达新建、＋/× 图标成组、拖拽调高、三态主题联动、扁平化等，参考 ZCode UI）+ AI 会话图标（Sparkles）与新建按钮图标化。分支 `bugfix/2026-08-app-shell-feedback` 共 17 提交，含延伸落地的工程质量治理第一档（CI clippy+fmt 闸门 + 前端 vitest 22 例）与设置中心第一阶段。任务目录已归档（`docs/work/archive/bugfix-2026-08-app-shell-feedback/`）。
