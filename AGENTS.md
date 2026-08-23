# AGENTS.md — Elwright

Elwright 是个人工作流工具箱：一个 Rust 共享核心 + 两个壳（CLI `ew` 现已可用；Tauri+Vue 桌面壳属阶段 3）。设计哲学「LLM 是增强不是地基」——核心离网可跑，skill 型在 LLM 不可达时降级为 SOP 文档而非报错。

## 目录

- `src-tauri/src/core/` — 共享核心：`registry.rs`（加载注册表/定位项目根）、`executor.rs`（按扩展名执行脚本）、`llm.rs`（OpenAI 兼容客户端）、`degrade.rs`（离线 SOP）、`invoke.rs`（LLM 调用/降级共享流程）。CLI 与桌面壳共用，勿把壳专属逻辑放进 core。
- `src-tauri/src/bin/ew.rs` — CLI 壳：`ls` / `run <id>` / `view <id>` / `invoke <id>`；`src-tauri/src/main.rs` — Tauri 壳与四个 IPC 命令。
- `src/` — Vue 3 桌面前端；组件仅依赖 `lib/bridge.ts`。
- `capabilities.json` — 内置注册表（顶层对象 `{capabilities:[...]}`，不是裸数组——曾按裸数组解析出过 bug）。当前只含三类能力各一个真实小示例（text-stats / capability-types / weekly-report）；个人能力走用户叠加层 `~/.elwright/`（导入按钮或 `ew import`），**不写回本文件**。
- `resources/tools/` — 脚本型能力的 .py/.ps1；`resources/docs/` — 知识型/SOP 的 .md + Agent 维护文档（AI_CODE_AGENT_*.md，被 AGENTS.md 引用勿删）。
- `docs/features/<feature>/` — 长期功能文档（README/behavior/architecture/changelog/decisions）；`docs/work/{active,archive}/` — 按维护方案组织的任务目录。
- `docs/ROADMAP.md` — **活文档路线图**：做什么/排在哪/状态，只看这里（架构方案 §10/§11 是立项规划快照，不再随进度更新）。
- `Elwright架构方案.md` — 权威架构文档。改架构相关代码前先读 §9（技术栈锁定决策）与 §12（阶段进度、Windows 工具链决策）。

## 构建与运行

```bash
# Rust 核心 + CLI
cd src-tauri
cargo build --bin ew          # CLI 二进制
cargo run --bin ew -- ls      # 或直接 target/debug/ew ls / view <id> / run <id> ...
cargo test                    # llm.rs URL 拼接等单元测试

# Tauri 桌面壳（先在 src/ 执行 npm install）
../src/node_modules/.bin/tauri build --debug --bundles app

# 桌面壳前端（src/ 是自包含 Vite 项目，package.json 在 src/ 内——有意为之）
cd src
npm install
npm run dev                   # 浏览器预览（dev 插件提供 /api/capabilities 与 /api/file 只读端点）
npm run build                 # 产出 src/dist/
```

CI：`.github/workflows/ci.yml` 在 push/PR 到 main 时跑三平台 `cargo test/build --bin ew` + `ew` 冒烟（含 mock LLM invoke 回归）+ 前端 `npm ci && npm run build` + 双平台安装包制品（macOS dmg、Windows msi，均由 CI runner 打包上传 artifact）。改核心或前端后本地先过这几条再推。

Windows 公司机器无 MSVC，用 GNU 工具链编译：`cargo +stable-x86_64-pc-windows-gnu build`，运行需 `D:\mingw64\bin` 在 PATH。只有 Tauri 桌面二进制被此卡住；CLI、reqwest、核心逻辑均不受影响。cargo 代理配置在用户级 `~/.cargo/config.toml`，不进仓库。

## 已锁定的技术决策（勿反复重议，见架构方案 §9.2）

1. LLM 客户端：自写 `reqwest` thin client 调 OpenAI 兼容 `/v1/chat/completions`，不引 `async-openai`。
2. 注册表 v1：纯静态 JSON，不做目录自动扫描。
3. 桌面 UI：Vue 3 + Vite。

## 约定与要点

- 面向用户的输出与错误信息用中文；代码注释中英混合均可。
- `capabilities.json` 字段：`id`/`name`/`type`（script|knowledge|skill）/`category`/`entry`/`doc`/`offline`/`prompt`/`degradeDoc`（JSON camelCase ↔ Rust snake_case，靠 serde rename）。新增能力 = 注册表加条目 + 文件放 `resources/` 对应子目录。
- `view` 命令优先读 `doc` 字段，回退 `entry`；知识型条目用 `doc`。
- `run` 仅限 script 型，`invoke` 仅限 skill 型（调用 LLM，失败/未配置降级 SOP）。
- 脚本执行按扩展名选解释器：`.py`→python3（探测回退 python/py，OnceLock 缓存）、`.ps1`→powershell、`.sh`→bash、`.bat/.cmd`→cmd，其他扩展名直接报错。资源工具脚本必须**纯标准库零依赖**（离网底线）。
- LLM 配置链（字段级合并，高→低）：环境变量 `ELWRIGHT_LLM_*` > 项目 `config.local.json` > 用户 `~/.elwright/config.json` > 注册表 `$meta.llmDefault`。CLI 用 `ew config` 查看/设置。LLM 客户端用 reqwest **blocking**（ADR-001，见 `docs/features/llm-invoke/decisions/`）。
- 资源根三段式解析（`registry::resolve_root`）：`ELWRIGHT_ROOT` env 覆盖 > cwd 上溯 > bundle 资源目录/exe 相邻探测。两壳都已接入；改根解析逻辑时同步跑 registry 单测。
- 正式打包：自 `src-tauri/` 跑 `../src/node_modules/.bin/tauri build`（`npx tauri` 在 `src/` 下找不到配置）。dmg 首次打包可能因 Finder AppleScript 超时（-1712）失败，重试即过。产物未签名。`resources/` 文件名保持 ASCII 且全局不重名（含隐藏占位文件）——WiX msi 打包对重名 basename 直接失败，macOS zip 会静默容错掩盖问题。
- 一键安装脚本（`install.sh` / `install.ps1`）从 GitHub Release 拉 dmg/msi 装到本机。**升级版本时记得同步 `install.ps1` 顶部的 `ProductCode`**——`file <msi>` 的 `Revision Number` 字段就是新版本的产品 ID，PowerShell 用它做"是否已装"探测；忘了改会导致同一台机器重复装两遍。脚本本身走 GitHub raw 域名分发，**任何改 commit 后即可用**，不需要发版流程。
- 本仓库在公司 Windows（走代理、GNU 工具链）与家里 macOS 双机共建，机器/网络特定配置勿写入仓库。家里 Mac 的 cargo 在 `~/.cargo/bin`（zsh 非登录 shell 可能不在 PATH）。
- `resources/docs/AI_CODE_AGENT_MAINTENANCE.md` 定义了本项目的 Agent 开发维护文档规则，新增功能文档时遵循它。

## Agent 工作协议（摘自 AI_CODE_AGENT_MAINTENANCE.md，细则以原文为准）

- **修改前**：读相关 Feature 文档（`docs/features/<feature>/`）与测试，总结当前行为；判断任务类型（feature/bugfix/enhancement/refactor/migration）。
- **修改时**：在 `docs/work/active/` 建对应前缀的任务目录（feature→plan/checklist/verification，bugfix→change-note/verification）；不往已完成的旧 Plan 追加新需求；行为变化同步更新测试与 Feature 文档。
- **修改后**：报告变更、测试与验证结果；按需更新 behavior/architecture/changelog/ADR；归档（active→archive）由人在确认上线后执行，Agent 不自行归档。

## 当前进度

看 `docs/ROADMAP.md`（活文档，唯一排期视图）。一句话现状：v0.1.2 + v0.1.3 hotfix 已发；`feat/chat` 合并到 main 进行中（合并后打 v0.1.4 tag 出 dmg + msi）。资源工具脚本约定：**纯 Python 标准库、零第三方依赖、中文报错**——离网可跑是底线。

## 前端约定（src/）

- UI 只依赖 `lib/bridge.ts` 的 `Bridge` 接口，禁止在组件里直接 fetch / 调 Tauri API。
- 预览模式下 run/invoke 有明确的降级文案（浏览器不能 spawn 进程），不要试图在浏览器适配器里"绕过"这一点。
- Markdown 渲染用 `marked` 直出 `v-html`：内容仅限本地 `resources/` 可信文件，不引入 sanitize 依赖；若未来渲染不可信来源必须重新评估。
