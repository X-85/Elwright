# 验证记录（2026-08-22，家里 macOS）

## 单元测试

cargo test：**26 通过 / 0 失败**。新增 3 例：

- `set_user_config_merges_and_clears_fieldwise` — 首次三字段写入；二次调用空值=清除对应字段、保留新 model。
- `set_user_config_preserves_unknown_keys` — 用户层文件里的 `custom_theme` 等未知键在保存后保留。
- `test_connection_reports_unreachable_in_chinese` — 连不上的端点报「无法连接 …」中文错误。

**竞态修复**：并行测试下 `merges_fieldwise_with_priority`（断言"无用户层时回退注册表默认"）与新的 `set_user_config` 测试（经 `ELWRIGHT_USER_ROOT` 写用户层文件）互相干扰——前者偶发读到后者写入的 key。修复：两者共用 `USER_ROOT_LOCK` 串行，且前者清 env 列表加 `ELWRIGHT_USER_ROOT`。连跑三次全绿确认稳定。

## 事故记录（已恢复）

首次 CLI 互通验证时误用了**改动前的旧 debug 二进制**：`ew config set` 不认识 `ELWRIGHT_USER_ROOT`，把测试值写进了真实 `~/.elwright/config.json`。已删除该文件并清掉空目录，重新构建后复验通过。教训：跑验证前先确认二进制是新的。

## CLI 互通（ELWRIGHT_USER_ROOT 指向临时根，新二进制）

1. `ew config set base_url http://127.0.0.1:18434/v1` → `已设置 base_url -> /tmp/.../config.json`（写入临时根，真实目录不动）。
2. `cat <临时根>/config.json` → 内容正确（`{"api_key":…,"base_url":…}`）。
3. `ew config` → base_url/model 来源标签「用户 ~/.elwright/config.json」，api_key 显示 `sk-c****`（打码）。

## mock LLM 全链路

本地 mock OpenAI 兼容端点（127.0.0.1:18434）下 `ew invoke weekly-report ping` → 返回 `pong`——`test_connection` 与 invoke 共用同一请求形态，链路等价验证通过。

## 前端构建

`npm run build` 通过（130.3 kB js，TS 类型检查含 `apiKey: string | null` 签名）。

## 浏览器 GUI（vite dev + in-app browser）

- 侧栏出现「⚙ 模型设置」按钮（与「＋ 导入能力…」并排）。
- 点开 → 弹层显示预览降级文案（浏览器不可读写用户配置，指引桌面/CLI）。
- 选 weekly-report → 调用 → 降级横幅出现「去配置模型 →」链接。
- 点链接 → 设置弹层打开。

## 待用户实机

桌面 app 上：⚙ 填真实端点 → 测试连接（应显示「连接正常」）→ 保存 → invoke weekly-report 走真实 LLM。
