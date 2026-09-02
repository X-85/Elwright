# Playwright 浏览器冒烟测试

## 目标

自动验证关键页面按钮和状态变化，不要求用户手工点击测试。

## 范围

- 引入 Playwright Chromium 测试和 Vite `webServer` 配置。
- 覆盖能力离线 SOP、收藏夹、课题报告、设置中心和工作区布局。
- 在 GitHub Actions 增加浏览器冒烟 job 与失败制品上传。
- 所有工作区数据使用隔离 `localStorage` 与虚拟文件路径。

## 非目标

- 不在浏览器测试中调用真实文件选择器、启动软件或读写用户文件。
- 不将浏览器预览结果视为 Tauri 终端、窗口 API 或 PTY 的验证。

## 验证

- 安装 Chromium 后运行 `npm run test:e2e`。
- 运行现有前端与 Rust 测试和生产构建。
