# Verification

## 自动化

- `quick_validate.py`：未能运行，当前系统 Python 与 Codex bundled Python 均缺少 `PyYAML`；已按脚本规则完成等价结构检查并通过。
- `python3 -m json.tool capabilities.json`：通过。
- `cargo fmt --check`：通过。
- `cd src-tauri && cargo test -q`：通过（41 tests）。
- `cd src && npm test -- --run`：通过（22 tests）。
- `cd src && npm run build`：通过。
- Codex Skill 结构检查：通过（必需文件、hyphen-case 名称、frontmatter、无 TODO）。
- `cd src-tauri && cargo run --quiet --bin ew -- ls`：通过，列出 `session-ledger`。
- 强制不可达 LLM 端点执行 `ew invoke session-ledger`：通过，正确降级显示本地 SOP。

## 手动验证清单

- `ew ls` 能列出 `session-ledger`。
- 未配置 LLM 时 `ew invoke session-ledger` 显示离线 SOP。
- 配置 LLM 后，输出明确说明 Elwright 不直接写入工作区，并给出 `session/` 台账指令。
- 在 Codex 中显式调用 `$session-ledger`，确认能创建或更新当前工作区台账；无写权限时不虚报。
