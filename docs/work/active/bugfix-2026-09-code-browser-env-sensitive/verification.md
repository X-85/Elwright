# verification：代码浏览器 .env 未被识别为敏感文件（B3）

## 修复内容

- `core::code_browser::is_sensitive` 覆盖 .env 全部命名变体（`.env`、`.env.local`、
  `app.env`、`config.env.prod`）。
- `search()` content 模式跳过敏感文件（snippet 不再泄露内容）。
- `build.rs` GNU 工具链测试二进制嵌 manifest（本机环境修复，CI 行为不变）。

## 验证记录

- 【自动化】独立 rustc 探针（GNU 工具链，2026-09-02 本机）：
  - 正例 10 项（.env/.ENV/.env.local/.env.production/app.env/config.env.prod/
    server.pem/id_rsa/credentials.json/db.password.properties）全部判定敏感 ✓
  - 反例 5 项（environment.ts/canvas.rs/App.java/provider.ts/dotenv.d.ts）
    全部不误判 ✓
  - 结果：ALL PASS
- 【自动化】单测 `is_sensitive_covers_env_variants` +
  `search_content_skips_sensitive_files` 已随代码提交；`cargo test --lib
  code_browser` 本机 GNU 无法运行（lib-test 目标不应用
  `rustc-link-arg-tests`，cargo 边缘行为，详见 change-note），**待 CI
  （Windows MSVC + macOS）执行**。
- 【自动化】`cargo fmt --check`：FMT-CLEAN（2026-09-02 本机 GNU）。
- 【自动化】`cargo clippy --lib --tests -- -D warnings`：0 警告（顺手修
  `executor.rs` 测试模块在 Windows 下 `#[cfg(unix)]` import 未用告警）。
- 【自动化】`cargo build --bin ew`（GNU）：成功。
- 【自动化】前端四闸（2026-09-02 本机）：eslint 0、vitest 15 文件 86 测试
  全过、coverage 91.72/75/98.98/96.84（阈值 70/60/70/70 达标）、vite build
  成功。本机 node_modules 曾因公司网络（SELF_SIGNED_CERT_IN_CHAIN/ETIMEDOUT）
  反复安装不完整，经内部 nexus 镜像 + 手工恢复 npm 半解包目录修复；
  jsdom 30 要求 node≥22，本机 node 20 以 --no-save 降为 undici@6 绕过，
  未改 package.json（CI node 版本满足，不受影响）。
- 【手测】（留给用户，装新 msi 后）代码浏览器 → 打开 `.env` → 预期提示
  「敏感文件只显示元数据」且内容为空；内容搜索 `.env` 中某密钥值 → 预期
  结果中不出现该文件。
