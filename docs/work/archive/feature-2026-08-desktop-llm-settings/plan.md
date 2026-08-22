# 桌面端模型设置（LLM 配置 UI）

## 目标

桌面壳补上 LLM 配置入口：此前只有 CLI `ew config` 能配，GUI 用户 invoke 失败只能看到降级文案，无处填端点也无从知道原因。

## 设计

- **只写用户层** `~/.elwright/config.json`——与叠加层同哲学（app 更新不丢、Windows 无权限问题），且与 `ew config` 同一文件，桌面/CLI 配置天然互通。
- **读**：复用 `ConfigLayers`（四级链合并），新增 `view()` 输出 `LlmConfigView`（生效值 + 每字段来源标签 + api_key 打码）。api_key 永不回传明文。
- **写**：`set_user_config(base_url, api_key, model)` 字段级合并，空值=清除字段，保留未知键。IPC 层 apiKey 用 `Option<String>`：`None`=保留现值（前端未动 key 时），`Some("")`=清除，`Some(v)`=写入——保留语义靠后端 `read_user_api_key()` 回填实现。
- **测试连接**：`test_connection` 发 max_tokens=1 的最小请求，完整走鉴权与响应格式校验，能真实反映 invoke 可用性。前端拿表单当前值测（未保存可测）。
- `user_config_path()` 支持 `ELWRIGHT_USER_ROOT` 覆盖（与叠加层同开关，测试隔离用）。

## 改动面

| 位置 | 改动 |
|---|---|
| `core/llm.rs` | `LlmConfigView`/`view()`/`set_user_config`/`read_user_api_key`/`test_connection`/`mask_key`；`user_config_path` 加 env 开关；3 个新单测 + 竞态修复 |
| `main.rs` | `get_llm_config`/`set_llm_config`(Option apiKey)/`test_llm_connection`(spawn_blocking) 三 IPC |
| `bridge.ts` | `LlmConfigInfo` 类型 + 三方法；浏览器 get/set 抛中文降级错误、test 直连 fetch |
| `LlmSettings.vue`（新） | 弹层：三字段表单（key 打码显示+留空不改）、来源标签、测试连接、保存 |
| `App.vue` | 侧栏「⚙ 模型设置」按钮 + 弹层挂载 |
| `CapabilityDetail.vue` | invoke 降级横幅加「去配置模型 →」链接（emit openSettings） |

## 验证

见 verification.md：26 单测全绿（含环境变量竞态修复）、npm build、CLI 互通（ELWRIGHT_USER_ROOT 隔离下 ew config set → core view 读回）、mock LLM invoke 全链路、浏览器 GUI（弹层/降级文案/引导链接）。
