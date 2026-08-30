# Plan · 工作工具栏第一阶段（Todo + 今日记录）

## 目标

交付 Workbench 第一阶段：**Todo 清单**与**今日记录（轻量记事本）**两个纯本地工具，作为主干闭环「记录工作上下文」的最小落地。

范围调整说明（2026-08-25 用户拍板）：原路线图 Workbench 第一阶段含收藏/最近使用、Todo、书签、高频转换工具，且「今日记录」后置。本次调整为 **Todo + 今日记录**先行，其余（收藏/最近使用、书签、转换工具）顺延为后续阶段。

## 范围

- **Rust core**（`core/workbench.rs`，复用 chat_store 模式：自由函数 + `registry::user_root()`）：
  - TodoStore：`~/.elwright/todos.json`，条目 `{id, text, done, createdAt, completedAt}`；操作 list/add/toggle/remove；进程内 Mutex 串行化文件读写
  - NoteStore：`~/.elwright/notes/YYYY-MM-DD.md` 一天一文件；get/save/list_dates；**日期格式严格校验**（`^\d{4}-\d{2}-\d{2}$`，防路径穿越）
- **IPC 命令 7 条**（`core/commands.rs`，同步命令 + `State<AppCtx>`）：`todo_list / todo_add / todo_toggle / todo_remove / note_get / note_save / note_list`
- **前端**：
  - `bridge.ts`：Workbench 类型与接口方法；tauriBridge 走 invoke；browserBridge 用**进程内模拟存储**（刷新即失，口径同 AI 对话会话预览）——UI 可在浏览器预览完整体验，持久化只在桌面壳
  - `WorkbenchView.vue`：Todo（输入/勾选/删除/计数）+ 今日记录（日期前后翻页、textarea 编辑、防抖自动保存、Markdown 预览切换复用 safeMarkdown）
  - `App.vue`：顶栏第三个导航入口（ListTodo 图标）
- **测试**：workbench 单测（含日期校验/往返）；`tests/workbench_ipc.rs` IPC 冒烟（同 terminal_ipc harness 模式）；vitest 补 browserBridge 模拟存储；Playwright 补工作台用例

## 非目标

- 收藏/最近使用、书签、高频转换工具（Workbench 后续阶段）
- AI 生成 Todo 草稿/对话内协作（AI 对话阶段③范畴）
- Todo 分组/标签/截止日、笔记全文搜索、富文本编辑
- CLI 侧 `ew` 子命令（先桌面，CLI 需要时另立任务）

## 基线与分支说明

自 `enhancement/2026-08-quality-tier2-e2e`（8f7be09）切出——依赖其命令层下沉（`core/commands.rs` + AppCtx）与 IPC 测试 harness。**合并顺序要求：tier2 先进 main（或本分支连带其提交一并合入）**。

## 风险与验证

- 并发写：todos.json 进程内 Mutex + 整文件重写（单用户桌面场景足够；无跨进程锁与 chats/ 口径一致）
- 路径穿越：note 日期参数严格正则校验，非法输入中文报错
- 验证方式：五道闸（cargo build/test/clippy/fmt + npm test/build/test:e2e）；【手测】桌面壳持久化（重启后 Todo/笔记还在、`~/.elwright/` 下文件正确）
