# 架构（Architecture）

```text
App.vue
  └─ SettingsCenter.vue
       ├─ 常规页
       ├─ 外观页 ── theme.ts ── localStorage + matchMedia
       └─ 模型设置页 ── LlmSettings.vue ── Bridge ── Rust ConfigLayers
```

- `App.vue` 只维护设置中心的打开状态，并在模型配置保存后刷新 `ChatView`。
- `SettingsCenter.vue` 管理当前分类；`LlmSettings.vue` 以嵌入模式复用现有模型表单，避免重复实现 key 脱敏、测试连接和保存语义。
- `theme.ts` 是唯一的主题状态入口：保存用户偏好、解析系统主题、设置 `document.documentElement.dataset.theme` 和 `color-scheme`。
- CSS 通过变量提供浅色和深色主题；业务组件继续引用既有设计变量。
- xterm 主题不在第一阶段接入，避免已创建的 `Terminal` 实例与 CSS 状态出现不同步。
