# Bugfix · 应用壳布局 + 图标改造真机反馈

基线：`feat/chat` HEAD（`5197a9f`，含应用壳顶部图标导航改造 + v0.1.4 chat 阶段）。
分支：`bugfix/2026-08-app-shell-feedback`

## Bug 列表（用户逐条补充）

| # | 现象 | 复现路径 | 预期 | 根因（Agent 填） | 修复（Agent 填） | 状态 |
|---|------|---------|------|----------------|----------------|------|
| 1 | 终端打开后能看到光标，但敲键盘看不到任何回显/输出 | 打开终端 → 点「＋ 新建」→ 敲任意键 | shell prompt 与输入回显正常显示，命令可执行 | `terminal_open` 用 `Channel::new(no-op)` 自建 channel 并塞进返回值；PTY 输出的 `channel.send(bytes)` 只调用本地 no-op 闭包，从未跨 IPC。前端 bridge 传的 `channel` 参数被 Rust 签名忽略。输入其实已写进 PTY（`terminal_write` 链路通），但 shell 回显随输出流进 no-op——前端零输出，看似「不能输入」 | Rust：`terminal_open` 增加 `channel: Channel<Vec<u8>>` 命令参数（Tauri `CommandArg` 自动解析回 JS onmessage 回调），返回值只带 `u64` id，删除自建 no-op channel。前端注释同步说明返回值只有 id（调用处本就只用 id，无需改动） | ✅ 已验证通过（2026-08-23 用户确认），已归档 |
| 2 | 新建第二个终端 tab 后不能输入（切回第一个正常） | 开 tab1 → 开 tab2 → 在 tab2 敲键 | 每个 tab 独立可输入 | TerminalPanel 用单个 `<TerminalView :session="activeTab.session">`，切 tab 时 Vue **复用同一组件实例**只换 prop；而 TerminalView 的 `onOutput`/`onExit`/`term.onData` 全部只在 `onMounted` 接线一次。第二个 tab 的按键写进**第一个** session（tab1 会莫名出字符），第二个 PTY 输出无人监听 → 看似不能输入。与主目录无关。更深层：单实例换 session 的结构还有三个固有缺陷——切 tab 屏幕内容残留、scrollback 互相污染、TUI 备用屏状态错乱 | **结构重写**（弃用先前的 wireSession 最小补丁）：每个 tab 一个独立 TerminalView 实例，`v-for + v-show` 显隐切换（VS Code/ZCode 同构）。接线回到「挂载一次、session 永不替换」的稳定形态；每会话独立 xterm 缓冲（隐藏 tab 后台输出不丢、切回即见）；`panel-body` 改 `position:relative` + 实例 `absolute inset:0` 堆叠；ResizeObserver 在 `clientWidth===0`（display:none 隐藏）时跳过 fit，重新显示时 RO 以真实尺寸再触发。删除组件内 label/rename/expose（归属面板层） | ✅ 已验证通过（2026-08-23 用户确认），已归档 |

## 终端交互优化（2026-08-23 第二批，参考 ZCode UI）

| # | 需求 | 实现 |
|---|------|------|
| O1 | 顶栏点终端按钮：下方直接弹出并新建一个终端，进入主目录 | App.vue `toggleTerminal` → TerminalPanel 新增 `toggleFromToolbar()`：无 tab 时 `openTab()`（cwd=主目录）并展开，有 tab 纯切换展开态。Bridge 新增 `homeDir()`（Tauri `@tauri-apps/api/path`，浏览器返回 null）；openTab 默认 cwd 从 `props.cwd`（应用目录）改为 homeDir，「在终端中运行」仍显式传应用 cwd（`ew run` 靠 cwd 上溯找注册表根） |
| O2 | 新建按钮「＋ 新建」改「＋」；关闭键 × | 模板文案改「＋」（title 提示保留"新建终端标签"），关闭键本就是 ×，tab label 的 `@mousedown.stop` 防拖拽误触 |
| O3 | 终端窗口允许往上拉 | 表头整体 `cursor: ns-resize` + `mousedown` 拖拽：`heightPct` 20–85vh 夹紧，拖动时全局 cursor/禁选中文本；移除旧 4px 小把手（`.resize-handle`） |
| O4 | 左侧 ▼ toggle 移到右侧并替换为 × | 表头改为 `[tabs…] [＋] [×]`（O5 迭代后最终布局）：右侧 ＋ = 新建 tab，× = 收起面板（会话保留，再点顶栏终端按钮恢复）；tab 自身 × 仍是关闭该会话。展开后无 tab 时收起路径只剩顶栏按钮（面板 header 随面板隐藏） |
| O5 | ＋ 移到右侧 × 前面，右侧成组两个按钮 | 最终布局 `[tabs…] [＋] [×]`；＋ 样式从块状按钮改为与 × 一致的轻量文本钮（hover 变亮），视觉成组 |
| O6 | 表头最左加「终端」标题；面板配色与主界面主题一致；默认标签名可辨识；＋/× 用图标 | ①表头 `[终端] [tabs…] [＋(Plus)] [×(X)]`；②面板/表头/tab 全部改用 CSS 变量（`--panel/--border/--text/--text-dim/--accent-soft/--bg`），跟随亮/暗主题；③默认标签名 `终端 1/2/3…` 递增（双击仍可重命名）；④＋/×/tab 关闭换 `lucide-vue-next` 图标（与顶栏同库同风格），24px 热区 + hover 圆角高亮 |
| O7 | 终端随三态主题（系统/深/浅）切换变色 | 配合另一会话的主题设置中心（`1d75359`）：theme.ts 导出响应式 `resolvedThemeRef`（applyTheme 时更新）；TerminalView 用它做 xterm 初始化主题 + watch 运行时切换 `term.options.theme`（xterm 即时重绘，不需重建会话）；外框背景从写死 `#000` 改 `var(--panel)` |
| O8 | 终端区扁平化（向 ZCode 看齐）：消除「内容区像凸出来」的观感 | 三个来源：①xterm 底色原用 `--bg`(#16181d)，与面板 `--panel`(#202329) 有色差圈→改 `#202329` 同值，表头与终端区同一平面；②表头 border-bottom 横线去掉（保留面板顶部 border-top 作为抽屉边界）；③非激活 tab 从 `--bg` 填充块改透明底（激活态保留 accent-soft 高亮） |

改动文件：`src/lib/bridge.ts`（homeDir 接口 + 双实现）、`src/components/TerminalPanel.vue`（toggleFromToolbar / 拖拽 / 按钮文案）、`src/App.vue`（顶栏按钮接新方法）。

## 根因链（Bug #1 详述）

Tauri 2.11.5 `ipc/channel.rs`：

- `Channel::new(closure)` 的 `send()` 只调本地闭包（channel.rs:296）——**不跨 IPC**。
- 只有作为命令参数反序列化的 `Channel`（`CommandArg::from_command`，channel.rs:300-316）经 `JavaScriptChannelId::channel_on(webview)` 拿到指向 JS `onmessage` 的 channel，`send()` 才 eval 前端回调。

v1 原始提交（562a1eb）即写反了方向，此后无人真机点过 GUI 清单（archive verification.md 中 GUI 段明确写「需要用户手点确认」，ROADMAP 却标了「真机验证通过」）。本次应用壳改造让用户第一次真打开了终端，暴露出来。

## 验证

每条修完后由用户在真机重走一遍复现路径，确认「预期」列行为出现。

## 归档

2026-08-23：两个 bug 与 O1–O8 全部经用户确认真机效果，任务随分支 `bugfix/2026-08-app-shell-feedback`（13 个提交，含延伸的工程质量治理第一档 `47219e7`）归档。验证记录见 verification.md。
