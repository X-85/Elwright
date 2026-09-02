# 架构

Playwright 的 `webServer` 启动 `vite` 于 `127.0.0.1:4173`。浏览器访问的是现有预览适配器 `browserBridge`：能力和本地 SOP 来自只读开发 API，工作区数据仅位于测试浏览器上下文的 `localStorage`。

CI 单独安装 Chromium 后执行 `npm run test:e2e`；失败时上传 HTML 报告、trace、截图和录像。该测试层不访问 Tauri IPC，因此不触及用户的 `~/.elwright/`、真实文件系统或本机软件。
