# 变更记录

## 2026-08-22（能力导入/导出/删除）

- 新增用户叠加层（overlay）`~/.elwright/{capabilities.json,resources/}`：桌面壳与 CLI 共享合并视图，同 id 用户条目遮蔽内置；文件解析双根（overlay 优先回退 bundle）。app 更新只动 bundle，用户导入的能力不丢，Windows 亦无 Program Files 写权限问题。
- 桌面 UI：侧栏「＋ 导入能力…」（id 冲突弹确认覆盖）、详情页「⬇ 导出」（保存对话框）与「🗑 删除」（仅自定义项）；列表/详情「自定义」徽标；操作 toast。
- IPC 新增 `import_capability`/`export_capability`/`delete_capability`，`list_capabilities` 下发 `origin` 字段；注册 `tauri-plugin-dialog`。
- 浏览器预览：导出走 Blob 下载（真实 bundle 格式）；导入/删除显示降级文案（浏览器不可写用户目录）。
- CLI 同步：`ew import` 改写用户叠加层（不再写仓库根）、新增 `ew delete <id>`、`ew ls` 增 SRC 列（user/-）。

## 2026-08-22（更新检查）

- 侧栏「检查更新」按钮：查 GitHub 最新 Release 与当前版本比较，新版本给「前往下载」入口（opener 打开系统浏览器）。

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
