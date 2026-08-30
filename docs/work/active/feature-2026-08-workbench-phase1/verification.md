# Verification · 工作工具栏第一阶段

按 checklist 逐项记录结果（自动化项附命令与输出摘要，手测项贴用户确认）。

## Step 2 Rust core

- `cargo test workbench`：**5/5 绿**——Todo 往返（id 递增/completedAt 维护/camelCase 落盘）、空文本与超长拒绝、Note 往返与倒序、路径穿越与非法日期拒绝（含合法边界闰日/年末）、损坏文件降级空列表。

## Step 3 IPC 命令

- `cargo test --test workbench_ipc`：**4/4 绿**——mock runtime 真协议：Todo 增查勾删往返（camelCase 字段断言）、未知 id/空文本中文报错、Note 存取与日期列表、非法日期拒绝。
- 全量 `cargo test`：42（lib）+ 6（terminal_ipc）+ 4（workbench_ipc）= 52 绿，无回归。

## Step 4 前端

- `npm run build` 通过（bridge 类型扩展无错误）。
- `npm test`：**26/26 绿**（含新增 workbench-bridge 4 例：模拟存储语义与 Rust core 同口径）。

## Step 5 e2e

- `npm run test:e2e`：**6/6 绿**（含新增工作台场景：Todo 添加→勾选划线→计数→删除→空态、今日记录输入→「已保存」徽标→Markdown 预览渲染、日期翻页清空、预览模式提示）。
- 勘误记录：bridge.ts 生成时模板字符串被转义坏（\` 变 \`），build 报 Invalid Unicode escape sequence，已修复并重验。

## CI

- 分支 push 后待合并 main 首跑回填（ci.yml 仅 main/PR 触发，分支直推不跑）。

## 手测项

- 桌面壳持久化冒烟：**已由 Agent GUI 点验完成（2026-08-30，真机 tauri dev + 辅助功能自动化）**
  - Todo：真实 UI 添加「桌面验证待办」→ IPC todo_add → `~/.elwright/todos.json` 落盘（camelCase、nextId 正确）→ UI 回读；勾选 → completedAt 落盘 → 计数「1 / 1 完成」
  - 今日记录读取：文件放置 → UI 切日期 → note_get 经真实 IPC 加载渲染完整；日期前后翻页正常
  - 持久化闭环：File→Close Window 正常退出 → 重启 → Todo（含勾选状态）与笔记完整保留
  - 终端：terminal_open 成功建会话（tab「终端 1」出现）
  - 工具边界（非产品缺陷）：textarea 的 AX 自动写值在 WKWebView 不生效，故「UI 键入笔记→自动保存」与「终端敲命令回显」两点未由 GUI 自动化覆盖；两者分别由 Playwright 浏览器用例（同组件代码）与 terminal_ipc.rs 真 PTY 用例覆盖，且写入通道与 Todo 完全相同（已真机验证）
