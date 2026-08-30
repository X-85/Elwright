# 架构（Architecture）

## 组件边界

```text
App.vue
├─ workspace-shell
│  ├─ 自定义应用标题栏（始终显示，窗口控制、品牌、左栏开关、终端、设置与右栏开关）
│  ├─ 左侧导航与顶部工具区（可隐藏）
│  ├─ 中间主工作区（始终保留）
│  ├─ 右侧上下文面板与顶部工具区（可隐藏，占位）
│  └─ 左右侧栏恢复边缘入口
├─ LlmSettings（弹层）
└─ TerminalPanel（按需展开的底部抽屉）
   └─ TerminalView（xterm.js 会话视图）
```

## 状态归属

- `App.vue` 管理左栏、右栏的显隐，以及左右栏顶部入口调用；左栏开关位于始终显示的自定义标题栏，并与品牌相邻。
- Tauri 桌面窗口使用 `decorations: false`，由 `@tauri-apps/api/window` 提供关闭、最小化、最大化和拖动能力。
- `src-tauri/capabilities/default.json` 显式授予四个窗口操作权限，和前端自定义标题栏 API 一一对应。
- 全屏切换额外使用 `core:window:allow-is-fullscreen` 与 `core:window:allow-set-fullscreen` 权限。
- `TerminalPanel.vue` 继续管理终端的 `expanded` 状态、tab 和 `TerminalSession` 生命周期。
- 左栏顶部终端入口通过 `TerminalPanel` 暴露的 `toggleExpand()` 操作终端，避免在壳层重复维护终端展开状态；浏览器预览中该入口为禁用态。
- `runCommand()` 仍由终端组件负责创建 tab 和写入命令，因此“在终端中运行”与左栏入口共享同一状态路径。

## 布局约束

- 壳使用紧凑应用标题栏 + 左/中/右三列 grid；左栏开关属于标题栏，业务工具仍嵌入左右栏顶部。
- 标题栏和工作区使用相同的左侧列轨道；标题栏左侧单独绘制 `border-right`，不使用整行 `border-bottom`。
- 标题栏采用左右两个操作区：左侧承载窗口、品牌、左栏和终端，右侧承载设置与右栏开关；工作区只保留业务内容。
- 绿色按钮布局菜单通过 Tauri `currentMonitor`、`setPosition`、`setSize` 和 `setFullscreen` 实现窗口布局。
- 左右栏通过 grid 状态类收缩，不能让隐藏栏继续占据主工作区空间。
- 窄窗口下左右栏改为覆盖式定位，避免主工作区纵向堆叠。
- 终端继续使用 `position: fixed` 的底部抽屉，不参与主工作区 grid；收起时高度为 0、不可见且不接收鼠标事件。
- 终端 DOM 在收起时保留，以维护 xterm 实例和 PTY 会话；展开时恢复可见性和交互。

## 兼容边界

本次仅调整 Vue 壳和 CSS，不修改 Rust terminal backend、Tauri IPC、bridge 接口或 xterm 渲染协议。
