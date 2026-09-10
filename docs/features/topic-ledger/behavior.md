# 行为

- 一个 Topic 表示一个人的工作目标、范围和完成标准；一个 Chat/Task 可以归属于一个 Topic，一个 Topic 可以跨多个 Chat/Task。
- 新 Topic 存在于 `session/topics/<topic-id>/`，由 `topic.md`、`events.md`、`decisions.md` 和 `handoff.md` 组成。
- Agent 每轮只读取索引、当前 Topic 快照、交接摘要和相关事件，不读取完整聊天历史作为默认入口。
- 目标、完成标准或交付物明显变化时，Agent 只提示可能需要拆分，不自动创建或关闭 Topic。
- 重大决定必须标记为“待确认”或“已确认”；事实必须区分已验证、未验证、失败和推测。
- 查看页只读本地 Markdown；它不会执行命令、调用网络或修改台账。
- 旧 `session/index.md`、`events.md`、`decisions.md` 继续有效，Topic 通过“关联问题”引用 Q 编号。
