# 变更日志（Changelog）

## 2026-08-23 · 第一阶段

- 新增设置中心：常规、外观、模型设置分类。
- 新增系统、浅色、深色主题偏好，并本地保存。
- 现有模型配置表单迁移为设置中心的“模型设置”页。

## 2026-08-31 · 模型档案（Q19，多套 LLM 配置切换）

- ADR-001 「模型档案」落地：`profiles: Map<name, LlmProfile>` + `activeProfile: string` 与既有 flat 字段共存兼容（v0.1.10 之前的 flat-only 用户配置继续生效，配置兼容是底线）。
- core::llm 新增 `LlmProfile` / `UserConfigFile` / `read_profiles` / `save_profile` / `delete_profile` / `set_active_profile` / `rename_profile` / `list_profiles` / `get_profile` / `is_valid_profile_name` / `normalize_profile_name`；`ConfigLayers::collect` 走 `UserConfigFile::to_flat_config`（active 命中→profile/否则回退 flat）；profile 文件写采用 tmp + rename 原子写。
- profile 名校验：仅小写字母/数字/-/_，长度 1-32，解析时统一 lowercase；保存/切换/重命名 错误返回中文。
- CLI 子命令：`ew config profile list|show|use|add|remove|rename`；key 脱敏（仅前 4 位）；当前激活项 `*` 标记；错误退出码 1。
- IPC：`llm_list_profiles` / `llm_get_active_profile` / `llm_set_active_profile` / `llm_save_profile` / `llm_delete_profile`；mock runtime 5 例（list 空 / save 可见 / set+get 同步 / delete 激活清空 / flat-only 兼容）。
- 前端 `Bridge` 新增 5 方法；`browserBridge` 5 stub 抛明确中文降级；`tauriBridge` 5 invoke 包装；`LlmSettings.vue` 顶部加档案下拉（★ 标记当前激活 + 「(flat 字段)」哨兵）+ 「+ 新建档案」按钮 + 已配置档案清单（含删除）；`lib/profileName.ts` 提供可单测的档案名校验函数。
- 测试：core 5 个新单测（profile 名规则 / 旧 flat 兼容 / active 命中与回退 / save-delete-use 闭环 / rename 保活 active）；vitest 4 例（profile 名校验 + normalize）。本地五道闸：cargo 97（72+5+3+1+5+6+4+1）、strict clippy、fmt、vitest 51、build 全绿。
