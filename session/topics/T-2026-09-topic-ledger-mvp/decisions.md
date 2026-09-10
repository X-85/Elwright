# Topic 决定

## 已确认

### D1 | Topic 与旧台账兼容
- 决定：Topic 使用 `session/topics/<topic-id>/`，不删除或改写旧 `session/index.md`、`events.md`、`decisions.md`。
- 理由：ZCode 已有台账 Skill，兼容比迁移更适合明天直接试用。
- 证据：当前仓库的 session-ledger 协议与用户现有使用方式。
- 确认日期：2026-09-10

### D2 | 文件协议优先
- 决定：Topic 使用普通 Markdown，不依赖 Elwright 数据库、云端服务或在线 LLM。
- 理由：可以在 ZCode、Codex 和其他 Agent 间携带，也满足离线可用底线。
- 证据：MVP 目标为跨工具复用。
- 确认日期：2026-09-10

### D3 | 查看页只读
- 决定：MVP 查看页只读取本地 Topic 文件，不自动写回或执行命令。
- 理由：降低安全和跨平台风险，让页面可以直接双击使用。
- 证据：Topic 的写入职责仍由 Agent Skill 和用户掌控。
- 确认日期：2026-09-10

## 待确认

### P1 | 默认 Skill 名称
- 建议：公司机先安装为 `topic-ledger`，保留旧 `session-ledger`；稳定后再决定是否原地替换旧命令。
- 影响：避免明天安装失败或覆盖现有可用 Skill。
