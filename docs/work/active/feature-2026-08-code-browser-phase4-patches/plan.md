# 阶段④方案：受控补丁编辑（Controlled Patch Apply）

> 路线图 V2 代码浏览器阶段④。决策依据：`docs/features/code-browser/decisions/ADR-001-phase4-scope.md`。

## 目标

把阶段③「发送到 AI」扩展到「AI 回复 → 用户审视 → 用户接受 → 应用到文件」。不引入 Java Language Server，不内置编辑模式；最小可用闭环：diff 识别 → 三栏预览 → 逐块接受/拒绝 → 单文件写入 → 快照可撤销。

## 用户故事

> 作为研发，我在 AI 对话里把一段代码发给模型，模型回复里夹带了修复建议。我希望直接点「应用到代码」预览增删行，确认后写回源文件；如果改错了，一键撤销回原状。

## 范围与边界

### 进入

- diff 识别：` ```diff ` 代码块（含或不含 @@ hunk 头）；只识别 unified diff 行（`+`/`-`/` `）；其它行当普通文本。
- 文件路径：diff 头 `--- a/path` `+++ b/path` 取后者；或模型前置 `// file: path` 行；缺路径则整块拒绝。
- 三栏预览：左边原文件（缩略，可滚动），中间 diff（红删绿增），右边新文件（缩略，可滚动）。
- 单块接受/拒绝：每个 hunk 一个按钮；至少接受一个才允许写入。
- 写入约束：
  - 目标文件必须在「当前打开的项目」根内（用现有 projectId 绑定）。
  - 敏感路径统一拒：`*.env`、`.ssh/`、`.aws/`、`*.pem`、`*.key`、`/etc/`、`*/node_modules/`、`*/target/`、`*/.git/`（黑名单扩展性预留）。
  - 写入前做内容指纹（已有实现就跳过，避免空写入）。
- 撤销：写入前先做整文件快照（路径 → 原始内容 + sha256），写入后写入 `~/.elwright/code-browser/applied-patches.json`；用户可在面板撤销，恢复原始内容。

### 不进入

- 内置编辑器（monaco / codemirror 多光标）
- 多文件跨项目补丁
- 自动保存 / 自动 commit / git 集成
- 多向合并（写入冲突时直接拒绝，让用户先用 IDE 解决）
- 行级 hunk 的语法校验

## 技术方案

### 后端（src-tauri）

新增 IPC：
- `apply_patch_preview(projectId, patchText) -> PatchPreview { hunks: [{file, oldStart, oldLines, newStart, newLines, raw}], warnings: [] }`
  - 不写文件，纯解析 + 当前文件内容快照。
  - 路径白名单/敏感黑名单校验在 Rust core 集中做。
- `apply_patch_apply(projectId, hunks) -> ApplyResult { applied: [file], skipped: [file], snapshotId }`
  - 写入前：每个目标文件读 → 校验指纹（如果提供了 expectedSha）→ 改写 → 落盘。
  - 写入后：把快照写入 `~/.elwright/code-browser/applied-patches.json`，返回 snapshotId。
- `apply_patch_revert(snapshotId) -> RevertResult { restored: [file] }`
  - 读快照 → 写回原内容 → 删除快照记录。

新增模块：`src-tauri/src/core/patch.rs`
- `pub fn parse_unified_diff(text: &str) -> Result<Vec<DiffHunk>, String>` —— 单测 4 类（合法 / 缺路径 / 二进制 / 多文件）
- `pub fn apply_hunks_to_content(content: &str, hunks: &[DiffHunk]) -> Result<String, String>` —— 单测 3 类（匹配 / 上下文漂移 / 空文件）
- `pub fn is_sensitive_path(path: &Path) -> bool` —— 单测 4 例（.env / .ssh / node_modules / target）
- `pub struct PatchSnapshot { id: String, project_id: String, entries: Vec<{path, original_content, sha256, applied_at}> }` —— 持久化 helper

新增命令集成：在 `src-tauri/src/core/commands.rs` 加 `apply_patch_preview` / `apply_patch_apply` / `apply_patch_revert`，注册到 `main.rs` 的 invoke_handler。

### 前端（src）

`src/lib/bridge.ts`：新增 `applyPatchPreview` / `applyPatchApply` / `applyPatchRevert` 三个方法，浏览器预览抛「请用桌面应用」。

`src/lib/patch.ts`（新）：diff 文本的解析（前端只用来识别模型回复里的 diff 块并渲染「预览按钮」，避免每条消息都走后端 IPC）；后端是权威，前端解析只为触发入口。

`src/components/ChatView.vue`：助手消息块若含 diff，渲染一个内联「预览」按钮；点击打开 `<PatchPreviewDialog>`（新建组件）。

`src/components/PatchPreviewDialog.vue`（新）：
- 三栏布局（原 / diff / 新），行号同步滚动。
- 每 hunk 一个复选框 + 接受/拒绝按钮（默认全部接受）。
- 文件路径与敏感标记醒目展示；写入按钮在所有接受项前禁用。
- 「应用」按钮 → 调 `applyPatchApply`；成功后弹「已应用 X 个文件；快照 ID: xxx（点击撤销）」。
- 撤销入口：常驻侧栏或面板顶部；记录当前会话所有未撤销快照 ID。

`src/components/CodeBrowserView.vue`：在代码书签/收藏面板下加一个「最近应用」小列表（点击可跳回文件并提示仍可撤销）。

### 测试

- core patch 单测：parse / apply / sensitive / snapshot round-trip 共 8 例
- IPC 集成测试：apply_patch_apply 走 mock runtime，加 `apply_patch_*` 三例
- 前端 vitest：patch.ts 解析 3 例 + 对话组件 diff 识别 2 例
- e2e：浏览器层不涉及（功能是桌面专属），依赖 CI 桌面覆盖
- 本地五道闸：cargo 全部 / 严格 clippy / fmt / vitest / build

## 依赖与约束

- 无新第三方依赖。diff 解析手写，规模与阶段①轻量符号索引同档（树状文本操作）。
- 不修改 capabilities.json（不是能力，是 IPC 命令）。
- 不破坏 ADR-001（能力调用显式确认）、ADR-002（不可信 Markdown 渲染——diff 在 ``` 块内仍走 safeMarkdown，但 ```diff 块在 ChatView 中分流到 PatchPreviewDialog，不再用通用 Markdown 渲染）。

## 风险

- 上下文漂移：模型给出的 diff 与实际文件内容不严格对齐 → 拒绝应用并提示「hunk N 上下文不匹配，请先刷新文件」。单测覆盖此类。
- 跨平台路径：Windows 反斜杠 / macOS 正斜杠，diff 文本通常用 `/`，按 POSIX 路径解析，写入时用 PathBuf 转换。
- 大文件快照：原内容整体写快照对超大文件（>10 MB）会很重 → 设上限 10 MB，超出拒收；用户先用 IDE。
- 并发：同文件多补丁叠加 → 写快照时校验指纹，若已被撤销/再改则拒。

## 实施顺序

1. core patch.rs：parse / apply / sensitive + 单测 8 例
2. IPC：apply_patch_preview/apply/revert + mock runtime 三例
3. 前端 bridge + patch.ts + PatchPreviewDialog 骨架（先只走浏览器错误降级）
4. ChatView 助手消息识别 diff 块 → 渲染预览按钮
5. 端到端联调：选中片段 → AI 回复 → 应用补丁 → 撤销
6. 本地五道闸 + PR

## 验证

待实施后回填。详见 `verification.md` 模板。

## 不属于本阶段

- 多文件引用追踪（阶段⑤候选）
- 三向合并
- 与 IDE 的实时双向同步
- 工程图 / 流程图与代码片段的更深联动（依赖工程图 MVP）