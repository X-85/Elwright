# 变更记录

## 2026-08-21

首次实现桌面壳前端（浏览器预览版）：

- Vue 3 + Vite + TS 脚手架（`src/` 自包含项目，依赖仅 vue + marked）。
- Bridge 抽象层：浏览器适配器 + Tauri IPC 预留挂接点。
- 三栏 UI：类型筛选/搜索侧栏、能力列表、分型详情（run/view/invoke）。
- Dev 只读 API：`/api/capabilities`、`/api/file`（限 resources/，防目录穿越）。
