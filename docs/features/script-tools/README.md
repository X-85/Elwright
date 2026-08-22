# 脚本型能力（script tools）

## 功能简介

`type: "script"` 的能力是离线可跑的本地脚本：经共享核心 executor 按扩展名选解释器执行，CLI `ew run <id>` 与桌面壳「运行」按钮走同一执行链路。LLM 不参与 script 执行路径。

内置注册表（`capabilities.json`）只保留示例条目；个人脚本能力走用户叠加层 `~/.elwright/`（`ew import` / 桌面导入按钮）导入，**不写回内置注册表**。

## 当前状态

可用（阶段 5 起上线，2026-08-22 随注册表精简改为仅含 1 条内置示例）。

## 内置示例清单

| id | 入口 | 功能 | 限制 |
|---|---|---|---|
| `text-stats` | `resources/tools/text-stats/text_stats.py` | 统计文件行数、字符数、中文字符数、英文单词数 | 仅支持 UTF-8 文本；中文字符按 CJK 统一表意区间计数 |

## 资源工具脚本约定

- **纯标准库、零第三方依赖**——离网可跑是底线。
- 面向用户的报错用中文（参数缺失、文件不存在、编码错误等由脚本自身校验）。
- `resources/` 下文件名保持 ASCII 且全局不重名（WiX msi 打包约束）。

## 相关文档

- 业务行为：[behavior.md](./behavior.md)
- 技术结构：[architecture.md](./architecture.md)
- 变更记录：[changelog.md](./changelog.md)
- 代码：`src-tauri/src/core/executor.rs`（执行器）、`src-tauri/src/bin/ew.rs`（run 命令）
