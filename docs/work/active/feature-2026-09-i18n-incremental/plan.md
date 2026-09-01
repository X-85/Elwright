# feature-2026-09-i18n-incremental — i18n 增量迁移（Q36）

> 接续设置中心 ADR-002 第 3 条遗留：「枚举选项标签、LlmSettings 内部文案、其余视图」。

## 范围（2026-09-01 本批）

1. 枚举选项标签：`STARTUP_VIEW_OPTIONS` label → `labelKey`（`startup.*` 双语键），
   SettingsCenter 渲染走 `t()`。
2. `LlmSettings.vue` 全量迁移（26 个 `llm.*` 双语键；含删除确认/切换/新建/删除结果文案，
   `{name}` 占位符由调用方 replace）。
3. 补齐 en 字典历史缺口：`settings.section.terminal`（此前缺失靠 zh 回退，被键集守卫暴露）。

## 后续增量（本批不做）

- 其余视图壳层文案：PeopleChatView / ToolboxView / TerminalPanel / WorkbenchView 等
  （均为 zh 源语言硬编码；建议逐视图增量，每视图一次提交）。
- 能力类型/成熟度档位枚举标签（Toolbox 徽标）。

## 门禁

eslint 0 / vitest（键集完整性守卫）/ vite build / Playwright e2e。
