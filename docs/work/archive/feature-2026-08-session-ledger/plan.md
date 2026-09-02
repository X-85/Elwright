# 会话台账基准技能

## 目标

将既有会话问题台账提示词产品化为 Codex Skill，并作为 Elwright 内置基准技能和离线 SOP 打包。

## 范围

- 创建 `~/.codex/skills/session-ledger/`，使 `$session-ledger` 可维护当前工作区 `session/` 台账。
- 增加 Elwright 注册表条目及离线 SOP。
- 明确 Elwright 的文本生成边界和 Codex 的文件维护能力。
- 更新 Feature 文档、路线图与验证记录。

## 非目标

- 不为 Elwright 增加自动文件写入、后台追踪或云端同步。
- 不删除原始 `session-ledger-prompt.md`。

## 验证

- Codex Skill 结构校验。
- 注册表 JSON 校验、`ew ls` / `ew invoke` 离线降级。
- Rust 测试、前端测试与生产构建。
