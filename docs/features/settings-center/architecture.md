# 架构（Architecture）

```text
App.vue
  └─ SettingsCenter.vue
       ├─ 常规页
       ├─ 外观页 ── theme.ts ── localStorage + matchMedia
       └─ 模型设置页 ── LlmSettings.vue ── Bridge ── Rust ConfigLayers
                ├─ flat 字段（base_url / api_key / model）
                └─ 档案 (Q19) ── profiles + activeProfile ──┐
                                                           │
core::llm::ConfigLayers::merged 每字段独立：                  │
  1. 环境变量 ELWRIGHT_LLM_*                                  │
  2. 项目 config.local.json                                  │
  3a. 用户配置 activeProfile 命中 profiles[name] ────────────┘
  3b. 否则回退 flat 字段
  4. 注册表 $meta.llmDefault
```

- `App.vue` 只维护设置中心的打开状态，并在模型配置保存后刷新 `ChatView`。
- `SettingsCenter.vue` 管理当前分类；`LlmSettings.vue` 以嵌入模式复用现有模型表单，避免重复实现 key 脱敏、测试连接和保存语义。
- `theme.ts` 是唯一的主题状态入口：保存用户偏好、解析系统主题、设置 `document.documentElement.dataset.theme` 和 `color-scheme`。
- CSS 通过变量提供浅色和深色主题；业务组件继续引用既有设计变量。
- xterm 主题不在第一阶段接入，避免已创建的 `Terminal` 实例与 CSS 状态出现不同步。

## 配置解析顺序（Q19 升级）

`core::llm::ConfigLayers::merged()` 每字段独立按下列顺序取第一个非空值：

1. `ELWRIGHT_LLM_BASE_URL` / `ELWRIGHT_LLM_API_KEY` / `ELWRIGHT_LLM_MODEL` env（永远最高）
2. 项目 `config.local.json` flat 字段
3. 用户 `~/.elwright/config.json`：
   a. 若 `activeProfile` 命中 `profiles[name]` → 用该 profile 的字段（profile 字段为空时回退到 flat 字段同位置）
   b. 否则回退 flat 字段
4. 注册表 `$meta.llmDefault`

激活 profile 仅参与步骤 3。所有 LLM 调用方（CLI 脚本能力、ChatView 流式、chat_completion、invoke skill）走同一 `ConfigLayers::merged()`，自动享受 profile 切换效果。
