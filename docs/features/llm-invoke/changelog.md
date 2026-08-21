# 变更记录

## 2026-08-21（补充）

6 个技能型能力的离线 SOP 全部导入 `resources/docs/`——此前 `degradeDoc` 指向的文件不存在，降级只会报「SOP 文件不存在」。降级业务行为不变（`show_sop` 逻辑未动），只是降级内容从缺文件变为完整 SOP。

## 2026-08-21

首次实现技能型 invoke 完整链路：

- `llm.rs` 占位替换为真实的 OpenAI 兼容 `/v1/chat/completions` 客户端（reqwest blocking，见 ADR-001）。
- `ew invoke` 由"无条件降级"改为"有 LLM 配置则调用，失败降级 SOP"。
- `ew invoke` 增加 skill 类型限定（此前任意类型均可 invoke）。
- 新增种子 SOP `resources/docs/tech-grill-sop.md`（此前所有技能型 degradeDoc 均指向不存在的文件）。
