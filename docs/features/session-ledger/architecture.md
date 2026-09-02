# 架构

Elwright 复用已有静态注册表和 `core::invoke`：`capabilities.json` 中的 `session-ledger` 将提示词作为 system message 发送给兼容 OpenAI 的 LLM；失败或未配置时，`core::degrade` 读取本地 SOP。该路径只有一次模型请求和只读 SOP 文件访问，不接触用户项目文件。

Codex Skill 是用户本机安装的独立目录：`SKILL.md` 说明读取边界、生命周期和回复格式，`references/protocol.md` 保存目录模板与归档细节。Codex 会话在启用 Skill 后，使用其现有文件工具对当前工作区的 `session/` 目录操作。

两端共享协议而不共享存储实现，避免 Elwright 在未获文件操作授权时越权修改项目。
