# 工作工具栏（Workbench）

## 定位

工作工具栏是 Elwright 面向日常工作的统一入口：一部分工具帮助用户记录和组织每天的工作，另一部分工具帮助用户快速完成高频的小任务。

它与现有能力工具箱形成分工：Todo、书签和每日记录属于产品内置的工作状态；进制转换、JSON/Java Bean 转换等属于内置实用工具；用户导入的复杂脚本和技能仍由能力注册表管理。

## 当前状态

**第一阶段已实现（2026-08-25，`feature-2026-08-workbench-phase1`）**：Todo 清单 + 今日记录（轻量记事本），顶栏第三导航入口。范围调整：今日记录由「后置」提前与 Todo 一并交付；收藏/最近使用、书签、高频转换工具顺延为后续阶段。

- 存储：`~/.elwright/todos.json`（顶层对象含 nextId，进程内 Mutex 串行化）+ `~/.elwright/notes/YYYY-MM-DD.md` 一天一文件；日期参数严格校验防路径穿越。
- IPC：7 条命令（todo_list/add/toggle/remove + note_get/save/list），core/commands.rs。
- 前端：WorkbenchView.vue 双栏（Todo 列 + 今日记录），浏览器预览走进程内模拟存储（刷新即失），桌面壳真实持久化。
- 测试：core 单测 5 例 + IPC 冒烟 4 例 + vitest 4 例 + Playwright 1 场景。

## 首批工具

- 工作记录：Todo 清单、书签、今日记录、最近使用和收藏。
- 实用工具：进制转换、JSON 格式化、Java Bean → JSON、JSON → Java Bean。
- 后续可评估：URL/Base64 编解码、时间戳转换、UUID、正则测试、Markdown 预览、文本差异对比；这些不因“常见”自动进入主干。

## 设计原则

- 常用工具打开即用，优先本地计算，不依赖 LLM 或网络。
- 工具页面统一布局和复制/保存交互，但不为了统一而把 Todo、书签强行包装成脚本能力。
- 工作数据写入用户层，不改动内置注册表和资源。
- AI 可以生成建议和草拟数据，但新增 Todo、书签或执行转换前需要用户确认。
- 桌宠只提供快捷入口和摘要，复杂编辑回到主窗口。

## 相关文档

- 工具明细路线图：[`docs/TOOL-ROADMAP.md`](../../TOOL-ROADMAP.md)
- 现有能力模型：`docs/features/desktop-ui/`、`src/lib/bridge.ts`
- AI 对话：`docs/features/chat/`
- 桌宠：`docs/features/desktop-pet/`
- 路线图：`docs/ROADMAP.md`
