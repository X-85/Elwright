# 桌面壳前端（Vue 3 + Vite）

## 功能简介

Elwright 桌面应用：Vue 界面通过 Bridge 调用浏览器预览 API 或 Tauri IPC，提供能力列表（筛选/搜索）和详情操作（脚本运行、知识阅读、技能调用/离线降级）。

## 当前状态

阶段 3 已完成（2026-08-21）：浏览器预览和 macOS debug Tauri `.app` 均已验证。正式跨平台发布、签名及 bundle 内资源路径属于阶段 4。

## 代码入口

- `src/main.ts` / `src/App.vue` — 入口与三栏布局（侧栏筛选 / 列表 / 详情）
- `src/lib/bridge.ts` — **核心边界**：`Bridge` 接口 + 浏览器/Tauri 适配器；`createBridge()` 按运行环境选择
- `src/components/CapabilityList.vue` / `CapabilityDetail.vue` — 列表与分型详情
- `src/vite.config.ts` — 含 dev-only `/api/*` 只读中间件（见 architecture）
- `src-tauri/src/main.rs` — Tauri 入口及 `list_capabilities`、`view_doc`、`run_script`、`invoke_skill` IPC 命令
- `src-tauri/tauri.conf.json` — 桌面窗口、前端构建钩子和 bundle 配置

## 运行方式

```bash
cd src
npm install
npm run dev      # 浏览器预览（http://localhost:5173）
npm run build    # 产出 dist/

# Tauri 桌面开发与 macOS debug app 打包
cd ../src-tauri
../src/node_modules/.bin/tauri dev
../src/node_modules/.bin/tauri build --debug --bundles app
```

## 相关文档

- [业务行为](./behavior.md)
- [架构说明](./architecture.md)
- [变更记录](./changelog.md)

## 相关测试

- 浏览器预览冒烟：`docs/work/active/feature-2026-08-stage3-desktop-ui/verification.md`
- Tauri IPC/打包验证：`docs/work/active/feature-2026-08-stage3b-tauri-shell/verification.md`
