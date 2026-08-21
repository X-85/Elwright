# Elwright

> 普通人也能用上的大模型红利 · LLM 是增强不是地基

`Elwright` = `El`（超人 Kryptonian 姓氏，星/光/家）+ `wright`（工匠）。
「超人家的造工具的人」——让没有 AI、或没钱开通 LLM 的人，也能锻造自己的超能力。

## 它能做什么

一个**个人工作流工具箱**，把你的定制化能力（脚本 / 知识 / 技能）统一登记、统一入口。

- **脚本型**：离网直接跑（文档搜索、Excel转md、部署脚本……）
- **知识型**：离网可看（踩坑笔记、协议学习……）
- **技能型**：接了 LLM 才解锁；没 LLM 时降级成 SOP 文档，不报错

工具自身**不依赖任何 LLM 或 agent 运行时**。LLM 只是增强插件。

## 双形态

- **CLI**：`ew`（终端 / 现场 / 低配机 / 开源普惠主入口）
- **桌面应用**：Tauri + Vue（给想要界面的用户）

> 跨平台：Windows（优先）+ macOS；Linux 后续。

## 当前状态

🚧 **阶段 2 完成**：LLM 客户端（OpenAI 兼容 `/v1/chat/completions`，自写 reqwest thin client）+ `ew invoke` 技能型调用 + 离线降级 SOP 已跑通（含单测与降级/成功路径验证，见 `docs/features/llm-invoke/`）。阶段 3 桌面壳（Tauri+Vue）待做。见 `Elwright架构方案.md` §12。

## License

MIT — 自由使用、修改、分发，让更多人受益。
