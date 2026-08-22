# Windows msi 改由 CI 打包（enhancement）

## 背景

stage4-release 的 design.md §3 原计划在公司机装 MSVC（约 4.5-5 GB）本地打包 msi。用户决策改为方案 A：用 GitHub Actions 的 windows-latest runner 打包（自带 VS/MSVC，Rust 矩阵在 windows-latest 一直编译通过即为证明）。公司机无需装任何东西。

## 目标

- push 到 main 自动产出 `Elwright_<version>_x64.msi`，上传为 artifact（与 dmg job 同模式）。
- 不改核心代码、不改 tauri.conf.json，仅新增 CI job。

## 实现步骤

1. `ci.yml` 新增 `msi` job：checkout → rust stable → rust-cache → node 22 → `npm ci`（src/）→ `tauri build --bundles msi`（自 src-tauri/，bash shell）→ upload-artifact `Elwright-windows-x64`。
2. 更新 dmg job 上方注释（原注释写「Windows msi 等公司机装 MSVC 后补」，已过时）。
3. AGENTS.md 进度行同步。

## 关键决策

- 构建步骤用 `shell: bash`：Windows 默认 pwsh 无法按相对路径解析 `node_modules/.bin/tauri`（无扩展名），Git Bash 直接跑 .bin 下的 sh shim，与 macOS 命令完全一致。
- WiX 由 tauri-cli 在 Windows 上自动下载，runner 可用，无需预装。
- msi job 与 dmg job 同触发条件（push/PR/dispatch 都跑），保持两平台制品产出对称。
- 产物未签名（与 dmg 一致）；WebView2 走默认 downloadBootstrapper，安装时按需下载。

## 非目标

- GitHub Release 挂产物（等版本号三处同步后另起任务）。
- 公司机本地 MSVC 构建（开发自用仍走 GNU 工具链，不受影响）。
- nsis 打包格式。

## 验证方式

- push 后 CI 全绿、`Elwright-windows-x64` artifact 含 .msi。
- 下载 artifact 检查文件名与体积合理（在 Windows 机器实际安装由用户确认）。

## 风险

- Windows release 首次构建较慢（无缓存约 10-20 分钟），仅影响 CI 时长。
- 公司机走代理拉 crate 的配置与 CI 无关（runner 网络直连）。
