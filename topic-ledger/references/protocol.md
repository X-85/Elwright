# Topic 台账协议 v1

## `topic.md`

```markdown
# Topic: 话题标题

- id: T-YYYY-MM-DD-short-name
- status: 处理中
- created: YYYY-MM-DD
- updated: YYYY-MM-DD
- owner: human

## 目标
一句话说明要解决什么。

## 范围
- 包含什么
- 不包含什么

## 完成标准
- 什么结果出现时可以结束这个 Topic

## 当前摘要
当前进展、已完成和下一步，控制在 3 至 8 行。

## 已验证
- [已验证] 事实、命令、测试或产出。
- [未验证] 仍需人工或真实环境确认的内容。

## 未决问题
- [ ] 需要人决定或后续验证的事项。

## 产出物
- `path/to/file`：用途或结果。

## 关联会话
- Chat / Task 名称或链接。

## 关联问题
- Q编号（可选，兼容旧 session-ledger）。
```

## `events.md`

只记录当前 Topic 的短事件，不复制聊天原文：

```markdown
# Topic 事件

### 2026-09-10 | 检查点
- 新信息：
- 方案：
- 实际结果：已验证 / 未验证 / 失败 / 推测
- 下一步：
```

## `decisions.md`

```markdown
# Topic 决定

## 已确认

### D1 | 决定标题
- 决定：
- 理由：
- 证据：
- 确认日期：

## 待确认

### P1 | 建议标题
- 建议：
- 影响：
```

## `handoff.md`

这是换 Chat 或换 Agent 时优先阅读的文件：

```markdown
# Topic 交接

- topic: T-YYYY-MM-DD-short-name
- generated: YYYY-MM-DD
- status: 处理中

## 一句话状态

## 已完成

## 已确认决定

## 证据与产出

## 未验证与风险

## 下一步

## 给下一个 Agent 的边界
- 先读哪些文件
- 不要做什么
- 哪些决定必须询问人
```

## 与旧台账兼容

- 不删除旧的 `session/index.md`、`session/events.md`、`session/decisions.md`。
- 旧 Q 编号继续表示问题；Topic 在“关联问题”中引用它们。
- 新 Topic 的事件不重复抄入旧 `events.md`，除非它同时是现有 Q 问题的一次处理。
- `topic.md` 是当前状态，`events.md` 是过程，`decisions.md` 是决定，`handoff.md` 是最小交接上下文。
