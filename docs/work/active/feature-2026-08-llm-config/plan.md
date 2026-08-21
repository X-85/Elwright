# LLM 配置持久化（ew config）

## 目标

模型配置从「只有环境变量」升级为分层配置链，提供 `ew config` 命令查看/设置，配置持久化到文件。

## 设计

配置来源优先级（高→低）：

1. 环境变量 `ELWRIGHT_LLM_BASE_URL/API_KEY/MODEL`（保持不变，排障用）
2. 项目根 `config.local.json`（.gitignore 已预留此文件名——原作者本就规划了）
3. 用户级 `~/.elwright/config.json`（安装后无仓库场景的主配置）
4. 注册表 `$meta.llmDefault`（默认本地 Ollama）

字段级合并：每个字段取最高优先级的非空值。

## CLI

- `ew config`：显示生效配置（key 打码）+ 各字段来源
- `ew config set <base_url|model|api_key> <值> [--local]`：写入用户级（默认）或项目本地
- `ew config clear [--local]`：删除配置文件

## 验证

- 单测：文件解析 + 字段级合并优先级
- 手动：set → config 显示 → invoke 走新配置；桌面壳经 invoke::invoke_skill 自动获得同一配置链
