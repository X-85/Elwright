# 验证记录

## 自动化（2026-08-31）

- core 单测 9/9：收藏/书签切换对称性、按行去重、持久化回读、上限。
- IPC 冒烟 1/1：favorites/bookmarks toggle 走真实协议（对称切换还原用户层）。
- vitest 31、e2e 10（无新增场景；发送到 AI 需桌面文件访问，浏览器端无路径）。
- 全量：cargo / clippy(-D warnings) / fmt / vitest / e2e / build 全绿。

## 人工验证

- 【待用户】真机点验：收藏与书签重开保留、AI 预填内容与选区一致。
