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

✅ **阶段 3 完成**：Vue 3 + Vite 界面既可浏览器预览，也已接入 Tauri 2 IPC（列表、文档、脚本执行、技能调用/离线降级）；macOS debug `.app` 已构建通过。正式跨平台发布、签名和 bundle 内资源路径属于阶段 4。此前阶段 1（Rust 核心 + CLI `ew`）、阶段 2（LLM 客户端 + 技能型 invoke + 离线降级）均已完成。见 `Elwright架构方案.md` §12。

## 快速开始（开发者）

```bash
git clone https://github.com/X-85/Elwright.git && cd Elwright

# CLI：列出/运行/查看/调用能力（脚本型与知识型开箱即用，无需任何 LLM）
cd src-tauri && cargo build --bin ew
./target/debug/ew ls

# 桌面界面：浏览器预览（或 ../src/node_modules/.bin/tauri build --debug 构建桌面 app）
cd ../src && npm install && npm run dev
```

想让「技能型」能力接入大模型（可选，其余能力不受影响）：见 **[LLM 配置指引](docs/release/llm-setup-guide.md)**——支持本地 Ollama（免费、数据不出机器）或任意 OpenAI 兼容云端端点；不配置时技能型自动降级为可照做的 SOP 文档。

## License

MIT — 自由使用、修改、分发，让更多人受益。
