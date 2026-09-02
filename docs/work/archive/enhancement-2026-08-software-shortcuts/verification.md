# Verification

## 自动化

- `cd src-tauri && cargo test`：通过（40 tests）。
- `cd src-tauri && cargo fmt --check`：通过。
- `cd src && npm test -- --run`：通过（22 tests）。
- `cd src && npm run build`：通过。

## 手动验证

- 浏览器预览可创建和展示图标化快捷方式；启动时显示桌面模式提示。
- 桌面端真实应用启动待真机确认（macOS `.app`、可执行文件和参数）。
