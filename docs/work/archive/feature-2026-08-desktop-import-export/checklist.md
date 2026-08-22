# Checklist

- [x] core：registry `load_merged`（`load_with_overlay`）+ origin 标记 + 双根文件解析 + `user_root()` + 单测
- [x] core：export.rs 导入目标根参数化（写 overlay，自动建目录）+ `delete_capability`（引用计数清理）+ 单测
- [x] CLI `ew import` 改走 overlay 语义；新增 `ew delete`；`ls` 增 SRC 列
- [x] Tauri：tauri-plugin-dialog + import/export/delete 三 IPC + capabilities 带 origin（serde flatten）
- [x] bridge.ts：三方法 + origin 字段；浏览器导出 Blob 下载、导入/删除降级文案
- [x] App.vue/组件：导入按钮、导出/删除按钮（仅自定义）、自定义徽标、冲突 confirm、操作 toast
- [x] `cargo test` 全绿（22 通过，含 4 个新增）
- [x] `npm run build` 通过 + 浏览器预览验证（加载/降级文案/导出 Blob 下载/toast/截图留档）
- [x] Tauri debug 构建成功（Elwright.app 启动正常）
- [ ] **用户实机 GUI 导入/删除确认**（材料已备：`~/Desktop/work-summary.elw.json`，步骤见 verification.md）
- [ ] CI 六 job 绿（推送后回填）
- [x] 文档：ROADMAP 登记、desktop-ui README/changelog 更新、AGENTS.md 目录与进度指针
