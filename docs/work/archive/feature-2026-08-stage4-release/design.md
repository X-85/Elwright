# 阶段 4：打包与发布 · 设计文档

> 状态：设计预研（2026-08-21）。实施等阶段 3b（Tauri 壳）合入后启动；实施时在本目录补 plan.md / checklist / verification。

## 1. 目标

- Windows 出 `msi`、macOS 出 `dmg`（架构方案 §7、§10）。
- 安装包自带 `capabilities.json` + `resources/`，用户装完即用，不依赖仓库或源码路径（§7「自带不依赖外部路径」）。
- 发布：GitHub Release + tag，README 面向非技术用户 + LLM 配置指引（`docs/release/llm-setup-guide.md` 初稿已备）。

## 2. 核心设计：资源路径解析（3b 留给本阶段的问题）

**现状（dev 模式）**：CLI 与桌面壳都用「从 cwd 向上找 `capabilities.json`」定位项目根——这要求在仓库内运行，打包后不成立。

**方案：三段式解析，优先级从高到低**

1. 环境变量 `ELWRIGHT_ROOT` 显式覆盖（排障/高级用户）。
2. cwd 向上查找（开发模式行为，保持不变）。
3. exe 相邻的资源目录（打包模式）：`<exe_dir>/resources` + `<exe_dir>/capabilities.json`。

**Rust 侧落点**：core 提供 `resolve_root(exe_path: Option<&Path>) -> PathBuf`；CLI 传 `std::env::current_exe()`，Tauri 壳传 tauri 的 resource resolver 结果（`app.path().resource_dir()`）。core 不依赖 tauri 类型——调用方先解析成普通路径再传入，保持「CLI 与桌面壳共用 core、core 零壳依赖」的边界（AGENTS.md 约定）。

**打包配置**：`tauri.conf.json` 的 `bundle.resources` 声明 `["../capabilities.json", "../resources/**/*"]`；Tauri 会把它们放进各平台包的资源位置（macOS 进 `.app/Contents/Resources`，Windows 进安装目录），与三段式解析的第 3 档对齐。

**注意**：脚本型能力的解释器依赖（python3 等）不打包——脚本型在用户机器上「解释器在则可用，不在则报明确错误」，与「离网可用」承诺一致；README 的系统要求里写清。

## 3. Windows 构建（MSVC）清单

依据架构方案 §12.2 的调研结论：

1. 装 **Visual Studio 2022 生成工具**（仅 Build Tools，不是完整 VS）：勾选「使用 C++ 的桌面开发」= MSVC v143 工具集（~1.7 GB）+ Windows 11 SDK（~1.3 GB）+ 共享组件，合计约 4.5-5 GB。
2. **安装位置指定 D 盘**（如 `D:\VSBuildTools`）——公司机器 C 盘仅剩 ~14.6 GB。
3. 装完 `rustup default stable-x86_64-pc-windows-msvc` 切回 MSVC 工具链（现有 GNU 工具链保留，CLI 仍可用它编）。
4. WebView2 Runtime 已预装（151.0.4129.101），无需分发。
5. 产物：`tauri build` 出 `msi`；公司机走代理拉 crate 需用户级 cargo config（已有，不进仓库）。

## 4. macOS 构建

- 本机（家里 Mac）Xcode CLT 已就绪，`tauri build` 直接可跑，出 `dmg` + `.app`。
- 签名/公证：个人项目暂不做 developer ID 签名——未签名 app 首次打开需右键→打开（Gatekeeper），README 写明操作。日后有需求再补 `codesign` + `notarytool` 流程。

## 5. 发布流程（一次性 + 每次发版）

一次性：GitHub 仓库公开（已公开）、LICENSE MIT（已有）、README 改写为面向非技术用户（挂 `docs/release/llm-setup-guide.md`）。

每次发版：

1. 版本号：`src-tauri/Cargo.toml` + `tauri.conf.json` + `src/package.json` 三处同步。
2. `cargo test` + `npm run build` + 双平台 `tauri build`。
3. 产物挂 GitHub Release（msi + dmg），tag 形如 `v0.1.0`。
4. Release notes 列：新增能力、修复、已知问题（用各 Feature 的 changelog 汇总）。

## 6. 风险与开放问题

- Windows 构建只能在公司机器做（家里无 Windows）——发布流程要把「双平台产物来自两台机器」写清楚，避免漏一边。
- `resources/` 中未来若有二进制工具（如 jar 反编译工具链），需评估体积与许可协议。
- 自动更新（tauri updater）暂不做，列为远期。
