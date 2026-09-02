# 验证记录

## 结果

- `cd src && npm run build`：通过（保留既有 chunk size warning）。
- `cd src && npm test -- --run`：通过，3 个测试文件、22 个测试。
- `cd src-tauri && cargo check --no-default-features`：通过。
- `git diff --check`：通过。
- Tauri 桌面端：需点击绿色按钮检查四种布局。
- Tauri 桌面端：将鼠标移入绿色按钮，确认布局面板出现；移入面板时保持显示，移出后隐藏，点击页面空白处关闭。
- Tauri 桌面端：逐项检查左/右/上/下半屏、填充、三列、四格、全屏和恢复窗口，确认窗口覆盖对应工作区区域。
- 半屏尺寸修正：使用显示器工作区逻辑像素，并临时放宽最小窗口宽度，避免 Retina 缩放下被 `minWidth: 960` 撑大。
