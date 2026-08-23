# verification: 检查更新 bug 修复验证

## 自动化测试

```text
cargo test --bin elwright tests::update_info -- 1 passed
cargo test version:: -- 4 passed
```

回归测试 `update_info_serializes_camel_case` 通过；显式删除 `rename_all` 验证它会失败（`updateAvailable` 取到 `Null`）。

## 手动验证（需家里 mac 上跑 v0.1.3 dmg）

1. 安装 v0.1.3 dmg（覆盖 v0.1.1 或 v0.1.2）。
2. 确认 About 仍显示 `Version 0.1.3 (0.1.3)`。
3. 点击「检查更新」按钮。
4. **预期**：下方显示「已是最新版本（v0.1.3）」——确认 IPC 链路通了。
5. 如本地版本低于 GitHub latest，应显示「发现新版本 v<latest>（当前 v0.1.3）」并提供下载链接。

## CI 验证

PR / push 后 `.github/workflows/ci.yml` 的 `cargo test` 会跑全量单测，新增的 `update_info_serializes_camel_case` 在保护范围内。

## 已确认不影响的范围

| IPC 命令 | 返回 struct | snake_case 字段 | 风险 |
|---|---|---|---|
| `check_update` | `UpdateInfo` | `update_available`, `release_url` | 已修 |
| `list_capabilities` | `Vec<CapabilityWithOrigin>` | 无（flatten 的 `Capability` 已手写 rename） | OK |
| `view_doc` / `run_script` | `ViewDocResult` / `RunScriptResult` | 无（单段字段） | OK |
| `get_llm_config` / `set_llm_config` | `LlmConfigView` | 已有手工映射 | OK（bridge.ts 端） |
| `export_capability` | `ExportBundle` | flatten 进 `Capability`（已 rename） | OK |

`get_llm_config` / `set_llm_config` 用的是 `LlmConfigView`（在 `core/llm.rs`），bridge.ts 端手工映射 `raw.base_url → baseUrl` 等。如果将来要加字段，建议同步在那侧加 `rename_all` 并去掉手工映射，避免双层定义。
