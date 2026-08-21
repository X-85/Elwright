# 变更记录

## 2026-08-21（阶段 4：打包）

- 资源根改为三段式解析：`ELWRIGHT_ROOT` 环境变量覆盖 > cwd 上溯（原行为）> bundle 资源目录/exe 相邻探测——桌面壳在 setup 期经 `resource_dir()` 解析一次（OnceLock 缓存），安装后的 app 不再依赖仓库路径。
- `tauri.conf.json` 增加 `bundle.resources`（map 形式）：capabilities.json 与 resources/ 打进包内 `Contents/Resources/`，实测无 `_up_` 层级。
- macOS 产出 `Elwright_0.1.0_aarch64.dmg`（8.2 MB，未签名）；挂载核验资源落位 + 卷内启动冒烟通过。

## 2026-08-21

完成阶段 3b Tauri 桌面壳接入：

- 新增四个 IPC 命令，复用 Rust registry、executor 与 invoke 核心。
- 脚本执行捕获 stdout/stderr，技能调用的 LLM/离线 SOP 流程由 CLI 与桌面壳共享。
- Bridge 自动识别 Tauri 环境，现有 UI 零组件改造；侧栏显示当前模式。
- 新增 Tauri 构建配置、跨平台图标和 macOS debug `.app` 验证。

此前的浏览器预览版：

首次实现桌面壳前端（浏览器预览版）：

- Vue 3 + Vite + TS 脚手架（`src/` 自包含项目，依赖仅 vue + marked）。
- Bridge 抽象层：浏览器适配器 + Tauri IPC 预留挂接点。
- 三栏 UI：类型筛选/搜索侧栏、能力列表、分型详情（run/view/invoke）。
- Dev 只读 API：`/api/capabilities`、`/api/file`（限 resources/，防目录穿越）。
