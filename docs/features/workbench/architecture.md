# 工作工具栏架构

## 分层

```text
WorkbenchView.vue / ToolView.vue
        ↓ WorkbenchBridge
Tauri IPC：工作数据、打开目标、工具执行
        ↓
用户层数据（todo/bookmarks/daily notes） + 本地工具模块

能力工具卡片 ─→ 现有 Bridge ─→ registry / executor / invoke
AI 对话 / 桌宠 ─→ 受控确认接口 ─→ WorkbenchBridge
```

- **内置工作数据**：Todo、书签、每日记录和工具收藏属于桌面壳的数据模型，保存到 `~/.elwright/` 用户层。
- **内置实用工具**：优先使用 Vue/TypeScript 的纯本地实现；需要复用 CLI 时再下沉到 Rust core 或标准库脚本。
- **可扩展能力**：复杂或用户自定义工具继续走 `capabilities.json` 和 overlay，不因工作工具栏出现而改变现有 `script/knowledge/skill` 契约。
- **Bridge 边界**：组件不直接 `fetch`、读写文件或调用 Tauri API；浏览器预览只提供本地内存/降级行为。

## 已实现结构（第一阶段，2026-08-25）

```text
WorkbenchView.vue（Todo 列 + 今日记录双栏）
        ↓ Bridge.todoList/todoAdd/... noteGet/noteSave/noteList
Tauri IPC：core/commands.rs 7 条同步命令（无 State，user_root 自解析）
        ↓
core/workbench.rs：自由函数 + 进程内 TODO_LOCK（读-改-写串行化）
        ↓
~/.elwright/todos.json（顶层对象 {nextId, todos[]}，camelCase）
~/.elwright/notes/YYYY-MM-DD.md（一天一文件，纯文本）
```

- 浏览器预览：browserBridge 进程内模拟存储（模块级数组 + Map），语义与 Rust core 同口径（vitest 锁行为）。
- 防抖自动保存用 saveToken 递增令牌：切日期/连续编辑时丢弃过期响应，防止旧内容覆盖新日期。
- 复用 `registry::user_root()`（ELWRIGHT_USER_ROOT 测试覆盖同生效）；复用 chat_store 的简易 ISO8601（不引 chrono）。

## 分阶段技术顺序

1. ~~工具栏导航、工具元数据、收藏/最近使用和用户层数据模型~~（第一阶段以最小形态实现：导航 + Todo + 今日记录；收藏/最近使用待后续）
2. ~~Todo、书签、今日记录及删除/导出边界~~（Todo + 今日记录已实现；书签与导出待后续）
3. 进制转换、JSON 格式化和 Java Bean/JSON 转换。
4. AI 的推荐与确认写入/执行接口。
5. 桌宠快捷入口、摘要和跨平台打开目标验证。

## 边界

- 工作数据默认不上传，不读取屏幕、剪贴板或系统活动。
- 转换工具默认不依赖网络和 LLM。
- 打开本地文件、执行能力或写入数据等动作必须有明确用户操作；模型输出不能直接作为命令执行。

## 第二阶段（2026-08-31，ADR-001）

- 常用能力：`lib/capabilityRecents.ts` 本地存储封装（收藏/最近使用、去重置顶、
  上限淘汰）；App.vue `select()` 时 `recordRecent`；WorkbenchView 经
  `open-capability` 事件跳转工具箱。
- 实用工具：`lib/convert.ts` 纯函数（JSON / Base64 UTF-8 安全 / 时间戳秒毫秒自动），
  中文报错带 `cause`；WorkbenchView 单区块多 tab 切换。
