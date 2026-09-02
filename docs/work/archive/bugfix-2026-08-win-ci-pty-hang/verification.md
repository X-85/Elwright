# Bugfix · Windows CI cargo test 挂起 · 验证

## 本机（macOS）

- 验证时间：2026-08-24
- 结果：`cargo fmt --check` / `cargo clippy --all-targets -D warnings` / `cargo test`（37 例）全绿；PTY 测试真实执行通过（非 CI 环境不跳过）。

## Windows CI

- 验证时间：2026-08-24
- 复现路径：推送后观察 main 的 CI run，`Rust core (windows-latest)` 的 `cargo test` 步骤应在可感知时间内完成（跳过打印 + 其余测试正常跑完）。
- 结论：✅ 通过。run 32732506896（cdef791）7/7 job 全绿，Windows cargo test 步骤正常完成（此前 2.5h 挂死的同步骤本 run 数分钟内结束），证实根因即该 ConPTY 测试。
