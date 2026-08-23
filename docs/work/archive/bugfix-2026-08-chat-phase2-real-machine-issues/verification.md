# 验证记录（bugfix-2026-08-chat-phase2-real-machine-issues）

日期：2026-08-23

## 单元测试（cargo test）

- 修复了 `core::llm::tests::merges_fieldwise_with_priority` 与 `set_user_config_*` 因宿主 `~/.elwright/config.json` 残留字段导致的环境串扰（测试本身的设计问题，不影响生产代码）。改为直接构造 `ConfigLayers { user: None }`，消除对宿主真实配置文件的依赖。
- 31+6 = **37 测试全过、0 失败**，连续两次跑稳定。

## 前端构建

- `npm run build` 通过；产物大小与阶段②相近。

## IAB 浏览器预览（http://localhost:5201）

- 主页 + AI 对话入口正常点击切换；进入后两栏布局、侧栏空态、预览降级文案均显示。
- 代码块复制按钮的样式变更（加深背景与 opacity）属于 CSS-only，前端构建已涵盖；真机需用户肉眼确认深色代码块上「复制」按钮可见。

## 真机验证（待用户在桌面应用复测）

四项目标（来自 verification-record.md）：

| # | 项 | 改动 |
|---|----|------|
| 1.1 | 未配置引导 | `refreshConfig` 增加 `console.info` 输出 baseUrl/model/source/state 便于排查（用户可清空 `~/.elwright/config.json` 后重启复测） |
| 1.2 | 保存表单不消失 | `LlmSettings.onSave` 成功路径新增 `emit('close')` |
| 1.3 | 代码块无复制按钮 | `.code-copy-btn` 加深背景（`rgba(30,34,42,0.85)`）+ 提高对比度 + 加 `z-index:1` |
| 2.3 | 重命名后被覆盖 | ChatView 增加 `userRenamed: Set<string>`：`commitRename` 写入该 id；`persistCurrent` 检测到 userRenamed 命中时保留原 title；`selectSession` 加载会话时若磁盘 title 与自动算出的不一致，自动记入 userRenamed（跨重启稳定） |

复测命令（同 verification-record.md）：`tauri dev`，按表勾选。