# Feature Plan — 设置中心 模型档案（多套 LLM 配置切换）

> 对应 ADR：`docs/features/settings-center/decisions/ADR-001-model-profiles.md`
> 承接：设置中心阶段一（v0.1.8 路径）+ ROADMAP V2「设置中心一期延伸」第一档
> 范围：**只做命名 profile 的 CRUD + 激活切换 + UI 控件**；API Key 加密 / 项目级 profile 共享 / 模型选择辅助不在本档

## 目标

- 用户能在「设置中心 → 模型设置」里维护多套 LLM 配置（命名档案），并在它们之间一键切换。
- 既有的 flat 字段（`base_url`/`api_key`/`model`）配置**继续生效**——用户不需要迁移。
- CLI `ew config profile ...` 子命令可独立完成切换、列出、增删。
- 所有 LLM 调用方（CLI 脚本能力、ChatView 流式、invoke skill）自动按激活 profile 路由，不需各调用方改代码。

## 不做

- 加密 / OS keychain（keytar 评估）—— 后续独立 ADR
- profile 之间共享 key / 继承 / 默认值覆盖—— 当前痛点仅是"切换"，非"组合"
- 自动检测可用模型—— 模型选择辅助属于另一档
- 项目级 `config.local.json` 支持 profile—— flat 字段 + env 已能覆盖项目级需求
- 设置中心其他分类改动（外观、终端等）—— 不在本档

## 任务拆解

### 0. 准备

- 建任务目录（本文）+ checklist + verification（姊妹文件）
- 在 `docs/features/settings-center/decisions/` 写 ADR-001-model-profiles.md

### 1. 后端 `core::llm` 扩展

- 新增 `LlmProfile { name, base_url, api_key, model }`
- 新增顶层配置读取：
  - `profiles: BTreeMap<String, LlmProfile>`（按 name 排序保证稳定）
  - `active_profile: Option<String>`
- `ConfigLayers::collect` 增项目 / 用户 profile 读取
- `ConfigLayers::merged` 在 step 3 优先用 `active_profile` 命中的字段，找不到则回退 flat（**核心兼容点**）
- 限制：profile 名正则 `^[a-z0-9_-]{1,32}$`，解析时按 lowercase
- 单测：profile 解析 / 切换 / 旧 flat 兼容 / env 高优 / 注册表默认兜底

### 2. CLI 子命令

- `bin/ew.rs` `config` 子命令扩展（沿用 clap 子命令）：
  - `ew config profile list`：列出全部 profile 名 + 当前激活（`*` 标记）
  - `ew config profile show [name]`：展示字段（key 脱敏）
  - `ew config profile use <name>`：设置 `activeProfile`，自动新建 profile（若 profiles 不存在）
  - `ew config profile add <name>` / `remove <name>` / `rename <old> <new>`
- 输出中文友好；key 只显示后 4 位
- 现有 `ew config` 不动；profile 子命令为新顶级子命令

### 3. IPC + Tauri 命令

- `llm_list_profiles() -> Vec<ProfileMeta>`（name + 是否激活 + 来源标签）
- `llm_get_active_profile() -> Option<String>`
- `llm_set_active_profile(name: String) -> ()`
- `llm_save_profile(profile: LlmProfile) -> ()`
- `llm_delete_profile(name: String) -> ()`
- 全部走 `ConfigLayers` 同款 atomic write（已有 `save_user_config` 模式可复用）
- main.rs 注册 5 个 IPC
- mock runtime 测试：list / use / save / delete / 旧 flat 兼容各一例

### 4. 前端

- `src/lib/bridge.ts`：5 个方法 + 浏览器预览 stub 抛明确降级
- `src/components/LlmSettings.vue`：
  - 顶部加"当前档案：[name ▾]"控件（下拉切换 + 新建）
  - 切换 → 保存 `activeProfile` → 刷新表单字段（仍从 ConfigLayers 读）
  - 表单底部新增"+ 新建档案"按钮（用当前字段值为初值）
- `src/components/SettingsCenter.vue`：无变化（继续嵌入 LlmSettings）
- vitest：`profileSwitch.test.ts` 覆盖关键 reducer / validator

### 5. 文档

- `settings-center/behavior.md`：模型设置章节加"档案管理"段落
- `settings-center/architecture.md`：架构图增 `profiles` / `activeProfile`；新增"配置解析顺序"小节
- `settings-center/changelog.md`：本期变更条目
- `settings-center/README.md`：分类与阶段表更新（"多配置档案" 从"后续" 挪到 "第一阶段" 已完成）
- ROADMAP §V2「设置中心一期延伸」首条标记完成 + 划掉

### 6. 收口

- 本地五道闸：cargo（含新单测 + 新 IPC 测试）/ strict clippy / fmt / vitest（含新 vitest） / `npm run build`
- 提交 + push + 开 PR + CI 7/7 全绿
- 合并后写 Q19 台账
- 设置中心阶段一旧任务目录（`feature-2026-08-settings-center-followup`）保持 active 不动；本档独立新目录

## 风险与回滚

- **配置破坏**：所有改动都保持 flat 字段兼容；解析顺序回退路径有单元测试覆盖
- **profile 名冲突 / 大小写**：解析时统一 lowercase；保存时若已存在返回明确错误（中文）
- **key 泄露**：脱敏与现有 `LlmSettings.vue` 保持一致；profile 删除前确认提示
- **回滚**：删除 IPC + 还原 `ConfigLayers::merged` 至旧版即可，旧 flat 配置继续可用