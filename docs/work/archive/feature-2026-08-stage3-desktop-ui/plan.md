# 阶段 3：桌面壳前端（Vue 3 + Vite，前端先行）

## 目标

在 `src/` 内搭起自包含的 Vue 3 + Vite 前端（架构方案 §12.3 既定路径：前端先行，浏览器预览，无需 Rust 工具链），覆盖 CLI 的四类操作：

- 能力列表（按类型筛选、关键字搜索）
- script：查看入口、传参运行（预览态展示说明）
- knowledge：渲染 Markdown 文档
- skill：invoke 展示结果或降级 SOP

## 实现步骤

1. `src/` 作为 Vite 项目根（package.json / index.html / vite.config.ts 就地摆放，保持架构方案的目录规划）。依赖仅 `vue` + `marked`，无组件库（pi-mono 极简）。
2. `lib/bridge.ts` 抽象层：定义 `Bridge` 接口（list/run/view/invoke），启动时探测环境选择适配器——浏览器适配器（走 `/api/*` dev 中间件读真实 `capabilities.json` 与 `resources/` 文档）或 Tauri 适配器（预留 `invoke()` IPC 挂接点，Rust 侧命令属后续任务）。
3. Vite dev 插件：`/api/capabilities` 与 `/api/file?path=`（仅允许 `resources/` 前缀，防目录穿越）两个只读端点，让浏览器预览读到**真实注册表数据**。
4. UI（全中文）：左侧栏（品牌/类型筛选/搜索）+ 中列能力列表 + 右侧详情面板（按类型给出对应操作）。
5. `npm run build` 产出 `dist/`；dev 服务器浏览器冒烟验证。

## 非目标（后续独立任务）

- Tauri 2 IPC 命令实现与 `tauri build` 打包（等 Windows MSVC 或直接在 Mac 上做）。
- script 在浏览器预览中真实执行（浏览器无法 spawn，预览态只展示说明；真实执行走桌面壳 IPC）。
- skill 在预览中真实调 LLM（预览态固定走降级 SOP 展示，验证降级 UI；真实调用走桌面壳）。
- 流式输出、多轮对话、配置持久化 UI。

## 风险

- Markdown 渲染 XSS：内容全部来自本地 `resources/` 可信文件，风险低；仍不引入 html sanitize 依赖，UI 文档中注明边界。
- `src/` 内嵌 Vite 项目非 Vue 官方默认布局，需在 README/文档写清构建命令。

## 验证方式

- `npm install && npm run build` 成功。
- dev 服务器启动后：`/api/capabilities` 返回 24 项能力；页面列表/筛选/搜索/详情/文档渲染正常（浏览器截图冒烟）。
