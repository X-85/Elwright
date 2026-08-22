# GitHub Actions CI 门禁

## 目标

push/PR 到 main 时自动验证双机共建 + 双 agent 并行开发下的主干健康：

- Rust 核心：三平台（ubuntu/macos/windows）`cargo test` + `cargo build --bin ew`。
- CLI 冒烟：`ew ls`（注册表可解析）、`ew invoke tech-grill`（降级 SOP 可读）、mock OpenAI 兼容端点的 invoke 成功路径（unix）。
- 前端：`npm ci` + `npm run build`（src/ Vite 项目）。

## 实现步骤

1. `.github/workflows/ci.yml`：`rust`（矩阵三 OS）+ `frontend` 两个 job；concurrency 取消同分支旧跑；权限最小化（contents: read）。
2. 冒烟断言用 grep（能力 id、SOP 标题、mock 回复标记）；Windows 可执行文件名带 `.exe` 用 `format()` 表达式统一处理，冒烟步骤显式 `shell: bash`。
3. mock LLM 服务器用 heredoc 内联（复用阶段 2 手工验证过的同名脚本），仅 unix 跑。

## 非目标（显式推迟）

- macOS dmg / Windows msi 制品产出：依赖阶段 3b 的 tauri 配置（main 上还没有），现在写必然无法验证；按阶段 4 设计文档在 3b 合入后加 `tauri-*` job。
- clippy/fmt 门禁：现有代码未过 clippy，加 `-D warnings` 会立刻红；列为后续加固项。
- 覆盖率、浏览器 E2E：远期。

## 风险

- Windows runner 上 bash 调 `ew`（无扩展名）不解析 → 用带 `.exe` 后缀的完整名，显式 bash。
- reqwest TLS 依赖：ubuntu runner 预装 libssl-dev，且 reqwest 0.13 默认走 rustls 系（阶段 2 编译日志可见），无 OpenSSL 缺失风险。
- workflow 无法本地端到端执行：各命令均已在本会话对 main 同内容验证过，YAML 本身做语法校验；首次真实运行为下次 push。

## 验证方式

- YAML 解析通过。
- 逐条复核 workflow 命令与 main 当前状态的匹配（能力数不硬编码，grep 用稳定子串）。
- 下次 push 后 Actions 首跑结果人工确认（记录进 verification）。
