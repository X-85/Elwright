# bugfix：资源收藏（workspace_create_resource）恒失败

## 现象

v0.1.6 真机冒烟（macOS debug .app + AX 驱动）发现：资源与课题页手动添加
收藏（网页链接 / 文字笔记）后数据不落盘、表单不清空、无错误提示。
Console 直调确认 `workspace_create_resource` 的 invoke Promise 永不落定
（响应序列化失败时 IPC 静默丢响应）。

## 根因

`workspace::Resource.id` 无 `#[serde(default)]`，而前端新建资源不携带
`id`（服务端生成）→ 反序列化报 `missing field 'id'`。浏览器 e2e 走
localStorage 模拟存储，不经此 IPC 接缝，故 9 个 e2e 场景全绿仍漏网。

## 修复

- `src-tauri/src/core/workspace.rs`：`Resource.id` 加 `#[serde(default)]`。
- 新增 `src-tauri/tests/workspace_ipc.rs`：以前端真实负载形态（无 id）
  走 mock-runtime 真协议回归：create → 服务端生成 id → delete → load
  确认清理（本测试写真实 ~/.elwright，结束前自清理）。
