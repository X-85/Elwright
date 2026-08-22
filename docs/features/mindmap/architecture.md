# 脑图架构

## 数据模型

```json
{
  "schema": "elwright-mindmap/0.1",
  "id": "mindmap-001",
  "title": "桌宠规划",
  "root": {
    "id": "root",
    "text": "桌宠功能",
    "children": [],
    "status": "open",
    "links": []
  },
  "updatedAt": "2026-08-22T10:00:00+08:00"
}
```

节点关联使用 `{type, id}` 形式，不把 Todo、能力或对话对象复制进脑图文件，避免产生两份状态。

## 调用关系

```text
MindMapView.vue
      ↓ MindMapBridge / WorkbenchBridge
Tauri IPC：脑图 CRUD、保存、导入导出
      ↓
用户层 mindmaps/ 文件

AI 对话 ─→ 草稿/差异预览 ─→ 用户确认 ─→ 脑图存储
节点关联 ─→ Workbench / Bridge ─→ Todo、书签、能力、对话
```

## 实现建议

- 编辑器优先评估 Vue 3 兼容的节点画布库，例如 Vue Flow；先验证节点层级、拖动、缩放和自定义节点，再锁定依赖。
- 脑图文件属于用户层，不写入 `capabilities.json` 或 `resources/`。
- 画布组件只处理视图和交互，文件读写、关联对象和 AI 请求都经过 Bridge。
- 浏览器预览可使用内存数据或只读降级；不能伪装成已经持久化到用户目录。

## 边界与风险

- 大脑图可能变得很大，需要后续限制单图节点数、保存频率和渲染性能。
- 节点关联对象可能被删除或重命名，必须支持失效关联的可见提示。
- AI 草稿合并需要稳定的节点 id 和差异模型，不能采用整图覆盖。
