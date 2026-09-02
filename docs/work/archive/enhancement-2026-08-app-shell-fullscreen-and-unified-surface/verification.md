# 验证记录

## 结果

- `cd src && npm run build`：通过。
- `cd src && npm test -- --run`：通过，3 个测试文件、22 个测试。
- `git diff --check`：通过。
- Tauri 桌面端全屏按钮：已接入 `isFullscreen` / `setFullscreen`，需在重启后的桌面窗口点击确认视觉效果。
