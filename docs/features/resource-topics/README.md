# 资源管理与课题

Elwright 的本地文件收藏与课题工作区。收藏夹以文件夹树整理本地文件路径（不复制文件），课题可引用这些文件及补充资源并生成报告。

实现位于 `src-tauri/src/core/workspace.rs`、`src-tauri/src/main.rs` 和 `src/components/WorkspaceView.vue`。数据保存在用户层 `~/.elwright/workspace.json`，不改写内置注册表。
