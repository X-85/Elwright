# 架构（Architecture）

## 数据模型（ADR-001 §D2）

> 早期草案（2026-08-22）为嵌套 JSON 树（`elwright-mindmap/0.1` schema + children 递归）；
> 实施时改为扁平 DFS 方案（下方），关联设计（`{type,id}` 引用不复制状态）保留给后续
> 多对象关联功能。

- `MindmapDoc { id, title, nodes: Vec<MindNode>, updatedAt }`
- `MindNode { id, text, parent, collapsed, convertedTodo }`；根 `parent=null` 恒为首节点。
- **扁平数组 + parent 指针，数组顺序 = DFS 文档序（子树连续）**——所有树操作维持该
  不变量：子树操作退化为连续区间处理，序列化稳定，渲染按序缩进。

## 模块

```text
src-tauri/src/core/mindmap.rs   纯函数树操作（5 单测）+ ~/.elwright/mindmaps/ 原子持久化
src-tauri/src/core/commands.rs  mindmap_list/create/load/save/delete（5 IPC）
src/lib/mindmap.ts              前端镜像纯函数（vitest 5 例；与 Rust 同一套不变量）
src/components/MindmapView.vue  大纲编辑器（键盘流/折叠/转 Todo/Todo 导入/自动保存）
```

## 关联

- 存储与 chats 一文件一资源同模式；转 Todo 复用 `todo_add` IPC（工作台）。
- 工程图 MVP（Mermaid/画布）顺延独立批次，数据模型独立无返工风险（ADR-001 §D1）。
