# 技能型 invoke 与 LLM 客户端

## 功能简介

`ew invoke <id>` 调用技能型能力：配置了 OpenAI 兼容 LLM 时由 AI 执行能力 prompt；LLM 不可达或未配置时降级展示 `degradeDoc` SOP 文档，不报错退出——这是「LLM 是增强不是地基」哲学的落地。

## 当前状态

已实现（2026-08-21，待用户确认后归档）。

## 代码入口

- `src-tauri/src/core/llm.rs` — `LlmClient`：配置读取、URL 拼接、`chat()` HTTP 调用
- `src-tauri/src/core/degrade.rs` — `show_sop()`：离线降级
- `src-tauri/src/bin/ew.rs` — `Cmd::Invoke`：调用与降级决策

## 相关文档

- [业务行为](./behavior.md)
- [架构说明](./architecture.md)
- [变更记录](./changelog.md)
- [ADR-001: 使用 reqwest blocking 客户端](./decisions/ADR-001-blocking-reqwest.md)

## 相关测试

- `src-tauri/src/core/llm.rs::tests` — chat_url 拼接（3 例）
- 手动验证：见 `docs/work/active/feature-2026-08-stage2-llm-invoke/verification.md`
