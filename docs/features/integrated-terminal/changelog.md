# 变更日志（Changelog）

## 2026-08-22 · 立项（v1 进行中）

- 新增 feature 目录与文档骨架（README/behavior/architecture/ADR-001）
- 决策：xterm.js + portable-pty + Tauri Channel 二进制流；预留 SSH 扩展点
- 状态：代码未提交，进度见 [feature-2026-08-integrated-terminal](../../work/active/feature-2026-08-integrated-terminal/checklist.md)

## 2026-08-23 · 应用壳入口调整

- 终端默认改为完全隐藏，不再保留常驻标题栏高度。
- 新增应用壳顶部终端入口，按需展开或收起底部抽屉。
- 收起终端时保留会话和 tab，能力详情的“在终端中运行”仍自动展开终端。
