# 设置中心（Settings Center）

> 状态：**常规 / 外观 / 终端 / 模型档案 已交付**（Q19 把模型档案落地）。设置中心统一承载桌面应用的本地偏好与模型配置，避免全局设置分散在工具栏和业务页面中。

设置中心采用左侧分类、右侧内容的结构。它只收录会影响 Elwright 日常工作流的配置，不以复制完整 IDE 偏好系统为目标。

## 分类与阶段

| 分类 | 第一阶段 | 后续 |
| --- | --- | --- |
| 常规 | 设置中心入口与基础说明 | 启动视图、自动检查更新已交付；语言待 i18n 基建 |
| 外观 | 系统/浅色/深色主题 | 密度、界面缩放已交付；无障碍偏好待排期 |
| 模型设置 | 复用现有 OpenAI 兼容模型配置 | ✅ 模型档案（多套 LLM 配置切换）已交付（Q19）；模型选择辅助待排期 |
| 终端 | 预留分类 | 字体、字号、滚动历史、主题联动已交付 |

## 代码位置

- `src/components/SettingsCenter.vue`：设置中心壳与分类导航。
- `src/components/LlmSettings.vue`：模型设置表单（含 Q19 档案下拉 + 新建档案按钮 + 档案清单删除），可嵌入设置中心。
- `src/lib/profileName.ts`：档案名校验 + 归一化函数（前端校验，后端权威）。
- `src/lib/theme.ts`：主题偏好读取、保存与系统主题跟随。
- `src/style.css`：浅色/深色设计变量和设置中心布局（含 Q19 `.profile-bar` / `.profile-list` / `.add-profile-modal`）。

## 相关文档

- [behavior.md](./behavior.md)
- [architecture.md](./architecture.md)
- [changelog.md](./changelog.md)
- [ADR-001 模型档案](./decisions/ADR-001-model-profiles.md)
- [当前任务](../../work/active/feature-2026-08-settings-center-model-profiles/plan.md)
