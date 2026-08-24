# Bugfix · Windows CI cargo test 挂起（ConPTY 测试）

## 现象

v0.1.5 发版后 main 的 CI run 32653634306：6/7 job 绿，`Rust core (windows-latest)` 的 `cargo test` 卡 2.5 小时无输出（后被后续推送的 concurrency 取消）。同 run 的 `Windows msi artifact` job 同为全量 Tauri 构建 4 分钟完成，排除冷编译慢；卡点为测试执行阶段。

## 根因分析

嫌疑主因：`src-tauri/src/core/terminal/local.rs` 的 `spawn_produces_expected_output` 在 Windows 上真实拉起 ConPTY + PowerShell，`child.wait()` 无超时。

- 终端 v1（含该测试）随 v0.1.4 合入 main，但 v0.1.4 时代的 Windows CI run 均被并发取消——该测试**此前从未在 GitHub Actions 服务会话里执行过**，本次是首次，即挂。
- GitHub Actions Windows runner 是无交互服务会话，ConPTY 在此类会话中的挂起是已知风险类型（无法完全确认，日志需管理员权限拉取；但修复方案在两种情形下都成立）。
- 版本 bump 致 rust-cache 键失效是真实存在的次要因素（该 job 确需冷编译），但编译最多数十分钟，2.5h+ 无测试输出只能用挂起解释。

## 修复

两层防御（`local.rs` 测试内）：

1. **CI 环境跳过**：`cfg!(windows) && env CI` 时跳过并打印说明——Windows CI 不再执行真实 ConPTY 冒烟；本机 Windows 保留覆盖。
2. **watchdog 兜底**：`wait()` 前起 30s 线程 sleep 后 `killer.kill()`——即使本机 Windows 或未来环境变化导致 PTY 异常，测试最多挂 30 秒后由 killer 解除，不再可能永久挂起。

非 Windows 路径行为不变（macOS/Linux CI 继续真实执行该测试）。

## 验证

- macOS 本机：`cargo test --lib` 31 例全过（含本测试真实执行 ok）；fmt/clippy 全绿。
- Windows CI：待本提交推送后观察 run（期望：跳过打印 + cargo test 步骤正常完成）。
