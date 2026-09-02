# Verification

## 自动化

- `cd src-tauri && cargo test`：通过（41 tests）。
- `cd src-tauri && cargo fmt --check`：通过。
- `cd src && npm test -- --run`：通过（22 tests）。
- `cd src && npm run build`：通过。

## 手动验证清单

- 待加入 `releaseTier: 2`、`unlockAfterUses` 的测试能力，确认默认隐藏、全部视图中锁定、达到总使用次数后解锁。
- 待在桌面壳确认前端本机存储刷新后仍保留“查看全部”偏好和使用次数。
