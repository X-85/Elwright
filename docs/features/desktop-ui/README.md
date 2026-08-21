# 桌面壳前端（Vue 3 + Vite）

## 功能简介

Elwright 桌面应用的 Web 前端：能力列表（筛选/搜索）+ 详情面板（脚本运行、知识阅读、技能调用/降级）。阶段 3 前端先行：浏览器可预览（读真实注册表数据），Tauri IPC 适配器已预留接口待阶段 3b 接入。

## 当前状态

浏览器预览版完成（2026-08-21）。Tauri 桌面壳接入（IPC 命令 + `tauri build`）属阶段 3b，未开始。

## 代码入口

- `src/main.ts` / `src/App.vue` — 入口与三栏布局（侧栏筛选 / 列表 / 详情）
- `src/lib/bridge.ts` — **核心边界**：`Bridge` 接口 + 浏览器适配器；Tauri 适配器挂接点在 `createBridge()`
- `src/components/CapabilityList.vue` / `CapabilityDetail.vue` — 列表与分型详情
- `src/vite.config.ts` — 含 dev-only `/api/*` 只读中间件（见 architecture）

## 运行方式

```bash
cd src
npm install
npm run dev      # 浏览器预览（http://localhost:5173）
npm run build    # 产出 dist/
```

## 相关文档

- [业务行为](./behavior.md)
- [架构说明](./architecture.md)
- [变更记录](./changelog.md)

## 相关测试

- 手动冒烟（dev API + 浏览器 DOM 验证）：`docs/work/active/feature-2026-08-stage3-desktop-ui/verification.md`
