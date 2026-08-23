# 架构（Architecture）

## 组件边界

```text
App.vue
├─ 顶部全局操作栏
├─ workspace-shell
│  ├─ 左侧导航（可隐藏）
│  ├─ 中间主工作区（始终保留）
│  └─ 右侧上下文面板（可隐藏，占位）
├─ LlmSettings（弹层）
└─ TerminalPanel（按需展开的底部抽屉）
   └─ TerminalView（xterm.js 会话视图）
```

## 状态归属

- `App.vue` 管理左栏、右栏的显隐，以及顶部入口调用。
- `TerminalPanel.vue` 继续管理终端的 `expanded` 状态、tab 和 `TerminalSession` 生命周期。
- 顶部终端入口通过 `TerminalPanel` 暴露的 `toggleExpand()` 操作终端，避免在壳层重复维护终端展开状态。
- `runCommand()` 仍由终端组件负责创建 tab 和写入命令，因此“在终端中运行”与顶部入口共享同一状态路径。

## 布局约束

- 壳使用顶部栏 + 工作区两行 grid；工作区内部使用左/中/右三列 grid。
- 左右栏通过 grid 状态类收缩，不能让隐藏栏继续占据主工作区空间。
- 终端继续使用 `position: fixed` 的底部抽屉，不参与主工作区 grid；收起时高度为 0、不可见且不接收鼠标事件。
- 终端 DOM 在收起时保留，以维护 xterm 实例和 PTY 会话；展开时恢复可见性和交互。

## 兼容边界

本次仅调整 Vue 壳和 CSS，不修改 Rust terminal backend、Tauri IPC、bridge 接口或 xterm 渲染协议。
