# 桌面壳「检查更新」按钮

## 背景

v0.1.0 已发布 GitHub Release（dmg + msi）。用户诉求：应用内要有更新入口，避免手动翻 Actions/Releases 页。

## 目标

- 侧栏「检查更新」按钮：查询 GitHub 最新 Release，与当前版本比较，展示「已是最新」/「发现新版本 + 前往下载」。
- 「前往下载」打开 Release 页（桌面壳走系统默认浏览器，官方 opener 插件）。
- 离网/网络失败：明确中文文案，不崩溃（「LLM 是增强不是地基」哲学同样适用于更新检查）。

## 非目标

- 自动更新（下载 + 替换二进制）——架构方案列为远期，tauri updater 届时评估。
- 后台轮询/启动时检查——只手动触发，离网友好且不耗 GitHub API 限额。

## 设计决策

1. **版本比较逻辑放 core**（`core/version.rs` 纯函数 + 单测），CLI 将来可复用；语义从宽（v 前缀、缺段、预发布后缀都容忍）。
2. **HTTP 查询放桌面壳 main.rs**（`check_update` IPC）：core 不依赖 GitHub 特定 URL；reqwest blocking（与 ADR-001 一致）+ 10s 超时，spawn_blocking 包裹。
3. **版本号真相源 tauri.conf.json**：Rust 用 `CARGO_PKG_VERSION`（构建时来自 Cargo.toml，发版时三处同步保证一致）；前端由 vite define 注入 tauri.conf 的 version（预览模式显示用）。
4. **浏览器预览同样可用**：GitHub API 支持 CORS，browserBridge 直接 fetch（与桌面行为一致）；openExternal 用 window.open。
5. opener 插件授权：`src-tauri/capabilities/default.json` 声明 `opener:default`（仅打开 URL）。

## 实现步骤

1. core/version.rs：normalize + is_newer + 4 组单测。
2. main.rs：check_update 命令（GhRelease 反序列化 → 比较 → UpdateInfo）；注册 opener 插件。
3. bridge.ts：UpdateInfo 类型 + checkUpdate/openExternal 双适配器 + compareVersions（语义同 core）。
4. App.vue + style.css：侧栏 update-box（按钮 + 消息 + 下载链接独立成行）。
5. vite.config.ts：__APP_VERSION__ 注入。

## 验证方式

- cargo test（version 单测并入总套件）。
- npm run build + 浏览器预览 GUI 实测两分支（真实 API「已是最新」；临时注入 0.0.1 验证「发现新版本」文案与链接）。
- cargo build --release（CI 打包路径）。
