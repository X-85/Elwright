# 行为

- `session-ledger` 是 release tier 1 的内置技能，CLI 可通过 `ew invoke session-ledger [附加输入]` 调用，桌面端从能力列表调用。
- 配置的 LLM 可用时，技能根据用户输入生成可交给具备工作区读写能力 Agent 的台账启动或续写指令。
- LLM 不可用时，按现有技能降级机制显示 `resources/docs/session-ledger-sop.md`，而不是报错。
- Elwright 不会自动创建、修改或归档用户项目内的 `session/` 文件，也不得表示已经完成这些操作。
- Codex `$session-ledger` 在当前工作区可写时才会创建并维护 `session/`；无写入能力时必须明确说明，并仅在回复末尾临时输出台账更新。
- 台账协议使用 `index.md`、`events.md`、`decisions.md` 和 `archive/`，以 `Q1`、`Q2` 等编号识别问题；状态范围和验证标记见 SOP。
