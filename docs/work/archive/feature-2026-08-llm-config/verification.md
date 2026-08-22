# 验证记录（2026-08-22）

- 单测：字段级合并优先级（项目文件覆盖 base_url/model，api_key 回退注册表默认；空场景全「未设置」）。
- CLI 手动：`ew config` 显示来源标注；`config set --local` 写入 config.local.json 后生效配置切换为项目层；`config clear --local` 后回退注册表默认。
- `ew invoke` 经 invoke::invoke_skill 自动使用同一配置链（桌面壳同）。
- 设计要点：api_key 显示打码；LlmConfig 字段 serde default（允许部分配置文件）。
