# Checklist — 设置中心 模型档案

> 与 `plan.md` 对应，逐条勾选；实施中遇到新项直接追加到对应分类末尾

## 0. 准备

- [x] 建任务目录 + plan / checklist / verification 三件套
- [x] 写 ADR-001-model-profiles.md
- [ ] ADR 评审（用户定向 → 合并到 main，PR 单独开）

## 1. 后端 `core::llm` 扩展

- [ ] `LlmProfile` struct
- [ ] `profiles` / `active_profile` 字段加入用户配置读取
- [ ] `ConfigLayers::merged` step 3 优先用 `active_profile`，回退 flat
- [ ] profile 名 lowercase + 正则校验
- [ ] 单测：profile 解析 / 切换 / 旧 flat 兼容 / env 高优 / 注册表默认兜底（≥5 例）

## 2. CLI 子命令

- [ ] `ew config profile list` / `show` / `use` / `add` / `remove` / `rename`
- [ ] key 脱敏
- [ ] 当前激活 `*` 标记
- [ ] 中文错误

## 3. IPC + Tauri 命令

- [ ] `llm_list_profiles` / `llm_get_active_profile` / `llm_set_active_profile` / `llm_save_profile` / `llm_delete_profile`
- [ ] main.rs 注册 5 个 IPC
- [ ] mock runtime 测试 5 例（list / use / save / delete / 旧 flat 兼容）
- [ ] cfg-gated origin URL（沿用 terminal_ipc 模式）

## 4. 前端

- [ ] Bridge 5 方法 + 浏览器 stub
- [ ] `LlmSettings.vue` 档案切换下拉 + 新建按钮
- [ ] `SettingsCenter.vue` 嵌入不变
- [ ] vitest `profileSwitch.test.ts` ≥4 例

## 5. 文档

- [ ] `settings-center/behavior.md`：模型设置章节增 "档案管理"
- [ ] `settings-center/architecture.md`：架构图 + 解析顺序小节
- [ ] `settings-center/changelog.md`：本期条目
- [ ] `settings-center/README.md`：分类表更新
- [ ] `ROADMAP.md` §V2「设置中心一期延伸」首条标记完成

## 6. 收口

- [ ] 本地五道闸全绿
- [ ] commit + push + 开 PR
- [ ] CI 7/7 全绿
- [ ] squash merge + 写 Q19 台账

## 实施期间追加

（实施中发现的新任务直接追加到这里；勾选后写入 verification.md）