# feature-2026-09-mindmap-mvp — 脑图 MVP

> 决策见 [ADR-001](../../features/mindmap/decisions/ADR-001-mindmap-mvp.md)（大纲式树形脑图；存储走 Rust core；Todo 双向关联）。

## 切片

### S1 core（Rust）
- `core/mindmap.rs`：MindmapDoc { id, title, nodes: Vec<MindNode>, updatedAt }
  - MindNode { id, text, parent: Option<String>, collapsed: bool, converted_todo: bool }
  - 操作纯函数：add_sibling / add_child / remove_subtree / move_up_down / indent_outdent
    （环防御：parent 链校验；id 生成走 unix 微秒+seq）
- 存储：`~/.elwright/mindmaps/<id>.json`，tmp+rename 原子写；list 按 updatedAt 倒序；
  损坏单文件跳过并在结果中标注
- IPC：mindmap_list / mindmap_create / mindmap_save / mindmap_delete
- 单测：树操作环防御、持久化 roundtrip、损坏文件容忍、删除子树级联

### S2 前端
- bridge：4 方法（浏览器预览降级）
- `MindmapView.vue`：大纲树（键盘流：Enter 兄弟 / Tab 缩进 / Shift+Tab 外提 /
  上下键导航）、折叠、节点转 Todo、侧栏「从 Todo 导入」、多图切换侧栏
- App.vue 侧栏入口「脑图」
- vitest：纯逻辑（若有抽取）；键集守卫兜底；e2e：预览降级守卫 + 桌面冒烟（如可行）

### S3 收尾
- behavior/architecture/changelog（features/mindmap/）+ ROADMAP 标记
- verification 回填 + 台账
