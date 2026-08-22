# 工程图架构

## 数据模型

```json
{
  "schema": "elwright-diagram/0.1",
  "id": "diagram-001",
  "title": "服务部署流程",
  "type": "flowchart",
  "graph": {
    "nodes": [],
    "edges": [],
    "participants": [],
    "messages": []
  },
  "mermaidSource": "flowchart TD\n  A[开始] --> B[构建]",
  "updatedAt": "2026-08-22T10:00:00+08:00"
}
```

流程图使用 nodes/edges，时序图使用 participants/messages；通用关联使用 `{type, id}`，不复制 Todo、能力或终端对象。

## 调用关系

```text
DiagramView.vue
      ↓ DiagramBridge / WorkbenchBridge
Tauri IPC：图形 CRUD、保存、导入导出、渲染
      ↓
用户层 diagrams/ 文件 + Mermaid renderer

AI 对话 / 脑图 ─→ 草稿差异 ─→ 用户确认 ─→ 结构化图模型
终端 / 服务器 ─→ 用户选择日志或关联对象 ─→ 工程图
```

## 实现建议

- 优先评估 Vue 3 兼容的画布库（例如 Vue Flow）用于流程图节点、连线、缩放和平移；不在未验证前锁定具体依赖。
- Mermaid 作为预览、导出和高级源码入口；结构化模型负责可视化模式编辑。
- Mermaid 子集解析和生成器独立于 Vue 组件；未知源码采用保留模式，防止用户代码丢失。
- 图形文件保存于用户层，不写入 `capabilities.json` 或 `resources/`；浏览器预览只提供内存或只读降级。

## 性能与失败路径

- Mermaid 语法错误显示行列和中文原因，保留上一份可渲染预览。
- 大图需要后续增加节点数、渲染频率和导出大小控制。
- AI 生成失败不影响原图；取消或拒绝差异后原图保持不变。
