# engineering-quality — 工程质量治理

跨功能的工程质量保障：CI 门禁、测试分层、验证清单约定。2026-08-23 因终端两个 bug（均落在保障空白区——IPC 接缝与前端组件）立项，分三档推进。

## 三档归属

| 档 | 内容 | 状态 |
|---|---|---|
| 第一档 | CI clippy + rustfmt；前端 vitest 覆盖纯逻辑模块（safeMarkdown / theme / bridge 纯函数） | 已完成（2026-08-23） |
| 第二档 | 分层 e2e 冒烟 + 验证清单标记约定 | 已完成（2026-08-24，`docs/work/active/enhancement-2026-08-quality-tier2-e2e/`） |
| 第三档 | eslint、覆盖率门槛 | 按需（eslint 低优先；覆盖率等测试有存量） |

## 测试分层（第二档确立）

```
┌─ 浏览器级（Playwright chromium，src/e2e/）────────────────┐
│  黑盒：browserBridge ↔ vite dev 插件接缝、预览模式降级守卫   │
├─ IPC 层（tauri mock runtime，src-tauri/tests/）───────────┤
│  真协议解析：CommandArg/Channel 走真实路径；macOS/Linux     │
│  真 PTY 全链路（open→write→exit→write 报错）              │
├─ Rust 单测（cargo test，core 各模块）─────────────────────┤
│  registry/llm/executor/chat_store/terminal 单元与集成      │
├─ 前端单测（vitest，src/lib/__tests__/）───────────────────┤
│  safeMarkdown 安全底线 / theme 三态 / 版本比较             │
└──────────────────────────────────────────────────────────┘
```

各层只测自己那层的接缝，不越层模拟：桌面壳真实 IPC 由 IPC 层覆盖，浏览器层不碰 Tauri API（与前端「UI 只依赖 bridge.ts」约定同构）。

## 验证清单标记约定

`checklist.md` / `verification.md` 条目必须带 `【自动化】`（附完整命令）或 `【手测】`（附最短复现路径）标记，约定全文见 `resources/docs/AI_CODE_AGENT_MAINTENANCE.md` §6。

## decisions

- [ADR-001-e2e-layering](decisions/ADR-001-e2e-layering.md) — e2e 选型：弃 tauri-driver，IPC mock + Playwright 分层
