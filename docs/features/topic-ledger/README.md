# Topic 台账

Topic 台账是跨 ZCode、Codex 和其他 Agent 的本地 Markdown 工作话题协议。它把人的工作边界、决定、证据和交接摘要从 Chat 中提取出来，避免换窗口或换工具后重新阅读完整上下文。

本仓库提供可单独交给 ZCode 安装的 `topic-ledger/` 源 Skill（Skill、协议和查看页）。它与现有 `session-ledger` 兼容，但不改写已有 Q 编号和旧台账文件；源 Skill 本身不依赖 Elwright 的 Rust 核心、桌面壳或注册表。

当前 MVP 不依赖 Elwright 桌面端或在线 LLM，明天可在公司 Windows 的 ZCode 中直接使用。
