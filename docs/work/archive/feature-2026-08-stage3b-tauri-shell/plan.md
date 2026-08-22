# 阶段 3b：Tauri 桌面壳接入（IPC + 打包）

## 目标

让 Vue 前端跑进真实桌面窗口，四类操作走 Rust 核心（与 CLI 同一份代码）：

- src-tauri 新增 Tauri 2 入口（`src/main.rs`）与 4 个 IPC 命令：`list_capabilities` / `view_doc` / `run_script` / `invoke_skill`，全部复用 `core/` 模块。
- 把 CLI `ew invoke` 的「调 LLM / 失败降级」流程下沉到 core（`core/invoke.rs`），CLI 与桌面壳共用——避免两处实现漂移。
- executor 增加「捕获输出」变体（桌面壳需把 stdout/stderr 展示到 UI；CLI 保留直通终端的交互能力）。
- 前端实现 tauriBridge 适配器（`@tauri-apps/api` invoke），`createBridge()` 探测 `__TAURI_INTERNALS__` 自动切换，UI 零改动。
- 本机 macOS 产出 debug `.app`（Xcode CLT 已就绪）；Windows 正式打包（msi）等 MSVC，属阶段 4。

## 实现步骤

1. `registry.rs`：`Capability` 加 `Serialize`（serde rename 双向生效，字段名与现有 JSON 一致）。
2. `executor.rs`：抽 `build_command`，新增 `run_script_capture(entry, args) -> ScriptOutput{code, output}`；CLI 的 `run_script`（直通终端）保留。
3. 新增 `core/invoke.rs`：`invoke_skill(root, cap, prompt) -> InvokeOutcome{source, content, note}`，ew.rs 改为调用它。
4. `src/main.rs`：Tauri Builder + 4 命令；项目根定位复用「向上找 capabilities.json」逻辑（cwd 找不到再从 exe 找）。
5. `tauri.conf.json`（frontendDist=../src/dist，devUrl=5173）+ 图标（生成 1024 源图 → `tauri icon`）+ `build.rs`。
6. 前端：`npm i @tauri-apps/api`，bridge.ts 加 tauriBridge；App 徽标按 bridge.kind 显示「预览模式/桌面模式」。
7. 验证：`cargo test`（含新增 executor 捕获测试）→ `npm run build` 回归 → `cargo build --bin elwright` → `tauri build --debug --bundles app`。

## 非目标（后续任务）

- 正式发布打包（msi/dmg、签名、资源内置打包）——阶段 4。
- 打包后 app 的资源路径解析（当前 dev 模式从仓库根读 capabilities.json；bundle 后 resources 需打进 app，阶段 4 处理并记录）。
- LLM 设置 UI（当前沿用环境变量）。
- Windows 侧构建（等 MSVC）。

## 风险

- Tauri 2 编译时间较长（全量依赖树）；Mac 上首次 build 可能数分钟。
- `tauri build` 对图标/配置完整性有硬校验——用官方 `tauri icon` 生成全套，避免手造 icns。
- 打包后的资源定位行为与 dev 不同：本次只保证 dev/debug 路径正确，bundle 资源方案在文档中显式记录为阶段 4 待办。

## 验证方式

- `cargo test`：executor 捕获输出单测（临时 .sh 脚本）+ 既有 llm 测试回归。
- `npm run build`：前端含 tauriBridge 编译通过（浏览器路径回归）。
- `cargo build --bin elwright`：Tauri 壳编译通过。
- `npx tauri build --debug --bundles app`：产出可打开的 `.app`（如环境不允许 GUI 验证，则以产物存在 + 编译通过为准，并如实记录）。
