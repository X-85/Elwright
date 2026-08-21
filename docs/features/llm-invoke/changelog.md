# 变更记录

## 2026-08-21

首次实现技能型 invoke 完整链路：

- `llm.rs` 占位替换为真实的 OpenAI 兼容 `/v1/chat/completions` 客户端（reqwest blocking，见 ADR-001）。
- `ew invoke` 由"无条件降级"改为"有 LLM 配置则调用，失败降级 SOP"。
- `ew invoke` 增加 skill 类型限定（此前任意类型均可 invoke）。
- 新增种子 SOP `resources/docs/tech-grill-sop.md`（此前所有技能型 degradeDoc 均指向不存在的文件）。
