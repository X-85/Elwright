# 桌面壳前端 —— 架构

## 模块职责

- `lib/bridge.ts`：UI 与数据/执行层之间的唯一边界。`Bridge` 接口定义四个操作（listCapabilities / viewDoc / runScript / invokeSkill）；UI 组件只依赖接口。
- 浏览器适配器（已实现）：list 走 `/api/capabilities`；view/invoke 降级走 `/api/file?path=`；run 返回预览说明。
- Tauri 适配器：`window.__TAURI_INTERNALS__` 存在时改用 `@tauri-apps/api` 的 `invoke()` 调 Rust 侧命令；IPC 错误转换为现有 UI 结果模型，避免组件感知平台异常。
- `src-tauri/src/main.rs`：注册 `list_capabilities`、`view_doc`、`run_script`、`invoke_skill`。后两项在 `spawn_blocking` 中运行，避免脚本与 blocking LLM 请求卡住窗口事件循环。
- `src-tauri/src/core/invoke.rs`：将 LLM 调用/离线 SOP 降级从 CLI 下沉为共享结果模型，CLI 与 IPC 均复用。
- `vite.config.ts` 内 `elwrightDataApi` 插件：dev 服务器上的只读中间件——`/api/capabilities` 返回真实 `capabilities.json`；`/api/file?path=` 返回 `resources/` 下文件（路径 resolve 后强制前缀校验，防穿越）。

## 调用关系

```mermaid
flowchart LR
    UI["App.vue / CapabilityList / CapabilityDetail"] --> B["Bridge 接口"]
    B -->|浏览器预览| HTTP["/api/capabilities · /api/file"]
    B -->|Tauri 桌面态| IPC["invoke() → Rust IPC 命令"]
    HTTP --> Files["capabilities.json · resources/"]
    IPC --> Core["registry · executor · invoke"]
    Core --> Files
```

## 关键设计

- **UI 不含数据访问代码**：全部经 Bridge，阶段 3b 换适配器时 UI 零改动。
- **预览即真实数据**：dev 中间件读仓库真实文件（非 mock），预览与桌面壳看到同一份注册表。
- Markdown 渲染用 `marked` 直出 `v-html`：内容全部来自本地 `resources/` 可信文件（与 CLI `ew view` 同源），不引入 sanitize 依赖。

## 系统边界

- 构建：Vite 8 + Vue 3 + TypeScript（仅 esbuild 转译，无 vue-tsc 类型门禁）。
- `src/` 是自包含 Vite 项目（package.json 在 `src/` 内，非仓库根）——这是有意为之，保持架构方案的目录规划。
- `tauri.conf.json` 的构建钩子从仓库根执行，因此以前端 `src/` 为相对路径；desktop debug bundle 当前仍依赖仓库根可被定位到。资源内置和发布 bundle 的资源路径是阶段 4 工作。
