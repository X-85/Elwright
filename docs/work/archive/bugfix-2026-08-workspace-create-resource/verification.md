# 验证记录

- `cargo test --test workspace_ipc`：1 passed（修复前同负载报
  `invalid args 'resource': missing field 'id'`，已用 scratch 测试抓到实锤）。
- 全量闸门：cargo 47+6+4+1 / clippy 0 error / fmt / vitest 26 / e2e 9 / build 全绿。
- 真机 UI 复验：未完成（显示器会话中断，GUI 路径不可用）；IPC 级已覆盖
  与前端完全相同的负载形态。待用户真机补一次「添加资源」点验。

## 人工验证（2026-08-31）

- 用户指示跳过真机复验，IPC 级回归已覆盖与前端相同的负载形态；留档待后续版本复验（见 PENDING-REAL-MACHINE-CHECKLIST.md A 节）。
