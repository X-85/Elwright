# Topic 台账 MVP

## 目标

把已有 session-ledger 从“Agent 的问题日志”扩展为“人的跨 Chat/Task 工作话题”，并产出可复制到 ZCode/Codex 的 Skill 与零依赖查看页。

## 范围

- 新增 `topic-ledger` Skill，兼容现有 `session/` 台账和 Q 编号。
- 新增 `topic.md`、`events.md`、`decisions.md`、`handoff.md` 协议。
- 新增可双击打开的静态查看页，选择 Topic 文件夹后显示人类可读状态。
- 提供 Windows PowerShell 安装脚本和使用说明。

## 非目标

- 不自动分析或拆分历史 Chat。
- 不修改现有 Chat 存储，不要求 LLM 才能查看 Topic。
- 不将现有“资源与课题”数据模型改名或迁移。
- 不做云同步、多人并发编辑或企业 Kanban。

## 验证

- Skill 结构和 YAML frontmatter 可被手工静态检查。
- PowerShell 脚本通过静态审阅；实际安装留给公司 Windows。
- 查看页能在兼容浏览器中选择 Topic 文件夹并渲染状态、决定、未决项和交接内容。
