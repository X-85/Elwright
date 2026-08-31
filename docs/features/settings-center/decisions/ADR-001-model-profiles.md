# ADR-001：模型档案——多套 LLM 配置命名切换（不破坏既有 flat 字段）

## 状态

已接受（评估决策，待实施验证）

## 决策

设置中心的"模型设置"分类升级为**命名档案（named profiles）**模型，并兼容既有 flat 字段的旧配置文件。具体两条：

1. **引入 `profiles: Map<name, LlmProfile>` 与 `activeProfile: string`**，用户可在「default / work / local-ollama」等多套命名配置间一键切换。
2. **不破坏既有 flat 字段（`base_url`/`api_key`/`model`）**——`~/.elwright/config.json` 仍兼容直接写这三个字段；profile 数据与之共存，profile 不存在时回退到 flat 字段行为。

理由与边界如下。

## 背景

- 设置中心阶段一（v0.1.8 路径之前）已交付设置中心壳 + 模型设置表单（嵌入 `LlmSettings.vue`），目前仅支持单一"现役"模型配置。
- ROADMAP §V2「设置中心一期延伸」列出"多配置档案与模型选择辅助"作为下一档。
- 现有 `core::llm::LlmConfig` 是 flat 三字段结构（`base_url`/`api_key`/`model`），由 `ConfigLayers::collect` 按 env > 项目 > 用户 > 注册表默认四层字段级合并；用户配置只允许一个有效配置。
- 实操中同一个用户会有多套 LLM：日常对话用云端大模型、本地脚本/小模型走 Ollama、临时调参要走 Azure OpenAI。**每次都得改 `~/.elwright/config.json`** 切换，URL/key 容易错位。

## 评估过程

### 候选方案

| 方案 | 描述 | 取舍 |
|---|---|---|
| **A. 命名档案（采纳）** | `profiles: { default: {...}, work: {...}, local-ollama: {...} }` + `activeProfile: "work"`。每次切换只改 `activeProfile`，profile 内容不动 | 切换快、命名直观、UI 可下拉；profile 名有限制避免滥用 |
| B. 多 `EnvVar` 切换 | `ELWRIGHT_LLM_PROFILE_NAME` 切换 + 同文件多 profile | 与现有 env 优先叠加层冲突；切换要重启 shell |
| C. 完全替换 flat 字段 | 移除 flat 兼容，只走 profile | 破坏现有用户配置；migration 成本高 |

### 风险

- **破坏性变更**：直接换掉 flat 字段会让现有 `~/.elwright/config.json` 全部失效，违反「离网即跑、配置兼容是底线」。
- **过度复杂**：支持"profile 之间共享 key"、"profile 继承"等高级特性会让配置链分支爆炸，与 Elwright "LLM 是增强不是地基" 的定位冲突。
- **激活切换的语义**：激活哪个 profile 必须有清晰语义——CLI 命令行覆盖 > `activeProfile` 字段 > env 变量。

## 决定

### 1. 数据模型

`~/.elwright/config.json` 新增两个**可选**字段（与既有 flat 字段并存）：

```json
{
  "base_url": "https://api.openai.com/v1",
  "api_key": "sk-...",
  "model": "gpt-4o-mini",
  "profiles": {
    "default":   { "base_url": "...", "api_key": "...", "model": "..." },
    "work":      { "base_url": "...", "api_key": "...", "model": "..." },
    "local-ollama": { "base_url": "http://localhost:11434/v1", "api_key": "", "model": "qwen2.5-coder:7b" }
  },
  "activeProfile": "work"
}
```

`LlmProfile = { name, base_url, api_key, model }`；profile 名限定 `[a-z0-9_-]{1,32}`，不区分大小写。

### 2. 解析顺序（`ConfigLayers::merged` 升级）

每个字段的来源从高到低（不变）：

1. `ELWRIGHT_LLM_BASE_URL` / `ELWRIGHT_LLM_API_KEY` / `ELWRIGHT_LLM_MODEL` env
2. 项目 `config.local.json`（flat 字段）
3. 用户 `~/.elwright/config.json`：
   a. 若存在 `activeProfile` 且能在 `profiles` 找到 → 用该 profile 的字段
   b. 否则回退 flat 字段（兼容旧配置）
4. 注册表 `$meta.llmDefault`

激活 profile 仅参与步骤 3；env 永远最高优。

### 3. CLI 与 IPC

- 新增 `ew config profile list|use <name>|show|add|remove <name>`（与现有 `ew config` 共用入口；输出当前激活、来源、可切换列表）。
- 新增 IPC：`llm_list_profiles` / `llm_get_active_profile` / `llm_set_active_profile(name)` / `llm_save_profile(name, ...)` / `llm_delete_profile(name)`，前端 `Bridge` 同步加方法。
- LLM 调用侧零变化：仍然走 `ConfigLayers::merged()`，所有调用方（CLI 脚本能力、ChatView 流式、chat_completion、invoke skill）自动享受。

### 4. UI

- `LlmSettings.vue` 在 flat 表单上方加一组「当前档案：xxx [切换▾]」控件：下拉显示全部 profile 名（含"未命名（用 flat 字段）"特殊项），切换后立即刷新表单字段并保存 `activeProfile`。
- 表单下方新增"+ 新建档案"按钮：把当前 flat 字段值作为初始值，让用户命名后写入 `profiles[name]`。
- 现有"测试连接 / 保存"语义不变；profile 名与字段一样在保存时校验。
- Key 字段沿用既有脱敏（仅前端展示后四位）。

## 影响的范围

- `core::llm.rs`：`ConfigLayers::collect` / `merged` 增 profile 解析；新增 `LlmProfile` 与 `active_profile_name()` / `effective_config_with_profile()` 辅助函数。
- `bin/ew.rs`：扩展 `config` 子命令解析（`list` / `use` / `show` / `add` / `remove`）。
- `core/commands.rs`：5 个新 IPC（profile CRUD + 切换）。
- `main.rs`：注册 5 个 IPC。
- `src/lib/bridge.ts`：5 个新方法；浏览器预览 stub 抛明确降级。
- `src/components/LlmSettings.vue`：加 profile 切换下拉 + 新建档案按钮。
- `src/components/SettingsCenter.vue`：无变化（继续复用 `LlmSettings.vue` 嵌入）。
- `src/lib/__tests__/`：vitest 增 `profileSwitch.test.ts` 等若干例。
- `src-tauri/src/core/llm.rs` 单测：profile 解析 / 切换 / 旧 flat 兼容 5 例。

## 拒绝的方案

- **完全替换 flat 字段**——破坏现有用户配置；违反"配置兼容是底线"。
- **profile 之间共享 key / 继承 / 默认值覆盖**——过度复杂；用户当前痛点只是"快速切换"，不是"配置组合"。
- **profile 加密（OS keychain）**——属于另一档（API Key 管理），本 ADR 不涉及。
- **profile 自动记忆"上次用的"**——已有 `activeProfile` 字段即等价实现。

## 验证

实施后回填：

- profile 解析 / 切换 / 旧 flat 兼容 5 个核心单测；
- `ew config profile` 子命令 5 例单测或手工；
- 5 个 IPC mock runtime 测试；
- 前端 vitest `profileSwitch.test.ts` 4 例 + Bridge stub 走通；
- 本地五道闸全绿；
- 真机点验：建 2 个 profile → 切换 → ChatView 头部模型名变化 → 流式与 invoke 都走新配置。

## 后续（不在本 ADR 范围）

- API Key 管理（OS keychain、keytar 评估）。
- profile 导出 / 导入 / 共享到项目 `config.local.json`。
- 模型选择辅助（自动 ping 给定 URL 列出可用模型）。