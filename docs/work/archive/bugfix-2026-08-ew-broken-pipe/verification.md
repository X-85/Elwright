# 验证记录（2026-08-21，家里 macOS）

## 复现（修复前）

- `bash -eo pipefail -c 'ew ls | grep -q doc-keyword-search'` → ew panic（Broken pipe），管道整体非零——与 CI run#2 macos 冒烟失败一致。

## 修复后

- `cargo build --bin ew` 成功；`cargo test` 6/6 通过。
- `bash -eo pipefail`（GitHub Actions bash 等价模式）下三条 CI 冒烟断言全部 PASS。
- `ew ls | head -3` 正常截断输出，无 panic。
- 常规输出（中文对齐、计数行）与修复前一致。

## 结论

修复有效。CI 冒烟在 macos/ubuntu 的失败因素已消除（本 bug）；ubuntu cargo test 失败的另一因素（Tauri Linux 系统库）由 workflow 加安装步骤处理，待 run#3 确认。
