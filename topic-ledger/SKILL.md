---
name: topic-ledger
description: Maintain human-readable work topics across ZCode, Codex, and other agents. Use when a conversation has a goal, decisions, files, verification, or a handoff that a person may need to resume later.
metadata:
  short-description: 跨 Agent 管理工作话题与台账
---

# Topic 台账

Topic 是人的工作单元；Chat 是交流容器；Task 是执行记录。本 Skill 在已有 `session/` 台账协议上增加一个可跨 Agent 携带的 Topic 快照，不替代原来的 Q 编号、事件和决定记录。

## 文件协议

优先使用当前工作区的以下结构：

```text
session/
├── index.md
└── topics/
    └── T-YYYY-MM-DD-short-name/
        ├── topic.md
        ├── events.md
        ├── decisions.md
        └── handoff.md
```

没有 `topics/` 时创建它，不覆盖已有的 `session/index.md`、`events.md` 或 `decisions.md`。旧台账继续有效；新 Topic 可以在 `topic.md` 的“关联问题”中引用 `Q编号`。

## 每轮工作

1. 先读 `session/index.md`，确定本轮归属的 Topic；再只读当前 Topic 的 `topic.md`、`handoff.md` 和相关 `events.md`。不要为了继续工作读取完整聊天记录或全部历史事件。
2. 如果当前 Topic 不存在，依据用户目标、范围和完成标准创建一个；不要把一次临时问答自动建成 Topic。
3. 开始工作时确认三件事：目标、范围、完成标准。目标或交付物明显改变时，先提示“可能需要新 Topic”，不要静默拆分。
4. 发生关键决定、文件产出、脚本执行或验证后，在 `events.md` 追加简短记录，并在 `topic.md` 更新当前摘要、状态和下一步。
5. 决定只有在人确认后才写入 `decisions.md` 的“已确认”部分；Agent 建议写入“待确认”，不要伪装成事实。
6. 话题结束或准备换 Chat 时，生成 `handoff.md`：成果、证据、未验证项、遗留问题和下一步。关闭动作需要用户确认。

## 状态和证据

状态沿用台账协议：`待处理`、`处理中`、`已解决`、`阻塞`、`搁置`。每条事实明确标记 `已验证`、`未验证`、`失败` 或 `推测`。不能把计划写成结果，也不能把 Agent 建议写成用户决定。

## 跨 Agent 交接

新 Chat、ZCode 或 Codex 只需先读当前 Topic 的 `topic.md` 和 `handoff.md`。Chat/Task 可以在 `topic.md` 的“关联会话”中追加名称或链接，但不要复制大段对话。Topic 文件必须是普通 Markdown，不能依赖某个客户端的数据库。

## 用户可见回复

有实际更新时，在回复结尾附：

```text
[Topic 更新]
话题：T-编号 / 标题
本轮：一句话总结
状态：当前状态
下一步：一句话总结
```

同时保留旧台账要求的 `[台账更新]`；如果本轮没有文件或状态变化，写“无新增 Topic 进展”。

## 参考资料

- 具体字段、模板和兼容规则见 [references/protocol.md](references/protocol.md)。
- 当前 Topic 的人类查看页是 `viewer/topic-viewer.html`；它只读取本地 Markdown，不会修改文件。
