# 验证记录

## 结果

- `cd src && npm run build`：通过（保留既有 chunk size warning）。
- `cd src && npm test -- --run`：通过，3 个测试文件、22 个测试。
- `git diff --check`：通过。
- 左栏展开 / 收起按钮位置：已移至始终显示的应用标题栏，位于窗口控制视觉组和 Elwright 品牌之后；左栏内部不再重复显示。
- Tauri 原生装饰：改为 `decorations: false`，自定义标题栏调用窗口 API；需在桌面端实际运行确认拖动和三种窗口操作。
- 标题栏事件：已显式调用 `startDragging()`，窗口按钮使用 `mousedown.stop` / `click.stop` 隔离拖动事件。
- Tauri ACL：补充四项 `core:window` 权限；重启 `tauri dev` 后不再出现 `window.* not allowed` 控制台错误。
