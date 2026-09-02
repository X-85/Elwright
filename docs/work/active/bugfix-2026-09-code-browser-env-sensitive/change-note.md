# change-note：代码浏览器 .env 未被识别为敏感文件（B3）

## 问题（真机点验 v0.1.13，Windows）

- 打开 `.env` 文件：内容完整显示，无任何敏感提示（`.pem` 正常拒绝）。
- 用户点验单：`docs/work/active/bugfix-2026-09-win-real-machine/real-machine-check-v0.1.13.md` B3。

## 根因

1. `core::code_browser::is_sensitive`（`src-tauri/src/core/code_browser.rs`）对 `.env`
   只匹配**点前缀**形态（`.env` / `.env.local`），不覆盖 `app.env`、
   `config.env.prod` 等「文件主名含 .env」的常见命名变体。点验项目里的 `.env`
   之所以漏网，与该函数当时的调用链一致；`.pem` 用的是 `ends_with` 所以正常。
2. **搜索泄露路径**：`search()` content 模式对所有文件读全文做匹配，
   敏感文件不过滤——`.env` 的密钥行会作为 snippet 直接出现在搜索结果里，
   这是比「打开文件」更隐蔽的内容泄露通道。

## 修改范围

- `src-tauri/src/core/code_browser.rs`
  - `is_sensitive`：增加 `== ".env"`、`starts_with(".env.")`、`ends_with(".env")`、
    `contains(".env.")` 四条规则，覆盖全部 .env 命名变体；其余黑名单不变。
  - `search()`：content 模式下 `is_sensitive(&name)` 的文件直接跳过（filename
    模式只显示文件名不泄露内容，维持原样）。
  - 新增单测：`is_sensitive_covers_env_variants`（含误判反例
    environment.ts / canvas.rs 等）、`search_content_skips_sensitive_files`。
- `src-tauri/build.rs`：顺手修复本机 GNU 工具链下测试二进制缺
  Common-Controls v6 manifest 的 ENTRYPOINT_NOT_FOUND（原 msvc-only 分支扩展
  到 gnu，用 windres 预编译 .res；windres 不可用时降级为 warning 不阻塞）。
  CI（MSVC）行为不变。

## 风险

- 低。黑名单收紧只影响代码浏览器的读取/搜索结果展示，不影响能力注册表、
  补丁编辑（`patch.rs` 的黑名单独立实现，本次未动）。
- `contains(".env.")` 理论上会误伤极少数以 `.env.` 中缀命名的普通文件
  （如 `some.env.backup`）——此类命名本就该按环境文件对待，接受。

## 验证

- 【自动化】`cargo test --lib code_browser`（CI Windows/macOS 跑；本机 GNU
  因 lib-test 目标不应用 `rustc-link-arg-tests`（cargo 已知边缘行为）无法
  运行测试二进制，修复逻辑已用独立 rustc 探针验证 15 正例 + 5 反例全过，
  门禁执行交由 CI）。
- 【自动化】`cargo fmt --check` + `cargo clippy -- -D warnings`。
- 【手测】真机复验：代码浏览器打开 `.env` → 应提示敏感不显示内容；
  内容搜索 `.env` 里的密钥值 → 不应出现在结果中。
