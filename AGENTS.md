# AGENTS.md — Elwright

Elwright 是个人工作流工具箱：一个 Rust 共享核心 + 两个壳（CLI `ew` 现已可用；Tauri+Vue 桌面壳属阶段 3）。设计哲学「LLM 是增强不是地基」——核心离网可跑，skill 型在 LLM 不可达时降级为 SOP 文档而非报错。

## 目录

- `src-tauri/src/core/` — 共享核心：`registry.rs`（加载 capabilities.json）、`executor.rs`（按扩展名选解释器 spawn 脚本）、`llm.rs`（阶段 2 占位）、`degrade.rs`（离线降级 SOP）。CLI 与未来桌面壳共用，勿把 CLI 专属逻辑放进 core。
- `src-tauri/src/bin/ew.rs` — CLI 壳：`ls` / `run <id>` / `view <id>` / `invoke <id>`。`find_root()` 向上查找 `capabilities.json` 定位项目根。
- `src/` — Vue 3 桌面前端（阶段 3，目前为空）。
- `capabilities.json` — 静态能力注册表（顶层对象 `{capabilities:[...]}`，不是裸数组——曾按裸数组解析出过 bug）。
- `resources/tools/` — 脚本型能力的 .py/.ps1；`resources/docs/` — 知识型/SOP 的 .md。注意：种子清单中许多 entry 是规划目标路径，文件尚未导入。
- `docs/features/<feature>/` — 长期功能文档（README/behavior/architecture/changelog/decisions）；`docs/work/{active,archive}/` — 按维护方案组织的任务目录。
- `Elwright架构方案.md` — 权威架构文档。改架构相关代码前先读 §9（技术栈锁定决策）与 §12（阶段进度、Windows 工具链决策）。

## 构建与运行

```bash
# Rust 核心 + CLI
cd src-tauri
cargo build --bin ew          # CLI 二进制
cargo run --bin ew -- ls      # 或直接 target/debug/ew ls / view <id> / run <id> ...
cargo test                    # llm.rs URL 拼接等单元测试

# 桌面壳前端（src/ 是自包含 Vite 项目，package.json 在 src/ 内——有意为之）
cd src
npm install
npm run dev                   # 浏览器预览（dev 插件提供 /api/capabilities 与 /api/file 只读端点）
npm run build                 # 产出 src/dist/
```

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
- 脚本执行按扩展名选解释器：`.py`→python3、`.ps1`→powershell、`.sh`→bash、`.bat/.cmd`→cmd，其他扩展名直接报错。
- LLM 配置读环境变量：`ELWRIGHT_LLM_BASE_URL` / `ELWRIGHT_LLM_API_KEY` / `ELWRIGHT_LLM_MODEL`；默认指向本地模型（如 Ollama `http://localhost:11434/v1`）。LLM 客户端用 reqwest **blocking**（ADR-001，见 `docs/features/llm-invoke/decisions/`）。
- 本仓库在公司 Windows（走代理、GNU 工具链）与家里 macOS 双机共建，机器/网络特定配置勿写入仓库。家里 Mac 的 cargo 在 `~/.cargo/bin`（zsh 非登录 shell 可能不在 PATH）。
- `resources/docs/AI_CODE_AGENT_MAINTENANCE.md` 定义了本项目的 Agent 开发维护文档规则，新增功能文档时遵循它。

## Agent 工作协议（摘自 AI_CODE_AGENT_MAINTENANCE.md，细则以原文为准）

- **修改前**：读相关 Feature 文档（`docs/features/<feature>/`）与测试，总结当前行为；判断任务类型（feature/bugfix/enhancement/refactor/migration）。
- **修改时**：在 `docs/work/active/` 建对应前缀的任务目录（feature→plan/checklist/verification，bugfix→change-note/verification）；不往已完成的旧 Plan 追加新需求；行为变化同步更新测试与 Feature 文档。
- **修改后**：报告变更、测试与验证结果；按需更新 behavior/architecture/changelog/ADR；归档（active→archive）由人在确认上线后执行，Agent 不自行归档。

## 当前进度

阶段 3 前端完成（Vue 3 浏览器预览版，见 `docs/features/desktop-ui/` 与架构方案 §12.5）。下一步：**阶段 3b** —— src-tauri 增加 IPC 命令（list_capabilities / view_doc / run_script / invoke_skill）+ tauriBridge 适配器（挂接点在 `src/lib/bridge.ts` 的 `createBridge()`，UI 零改动）+ `tauri build` 打包（Windows 需 MSVC，macOS 本机 Xcode CLT 已就绪）；其余技能型 SOP 文档批量导入。

## 前端约定（src/）

- UI 只依赖 `lib/bridge.ts` 的 `Bridge` 接口，禁止在组件里直接 fetch / 调 Tauri API。
- 预览模式下 run/invoke 有明确的降级文案（浏览器不能 spawn 进程），不要试图在浏览器适配器里"绕过"这一点。
- Markdown 渲染用 `marked` 直出 `v-html`：内容仅限本地 `resources/` 可信文件，不引入 sanitize 依赖；若未来渲染不可信来源必须重新评估。
