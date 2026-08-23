# bugfix: 检查更新按钮始终显示「已是最新版本」

- 任务：bugfix-2026-08-check-update-no-prompt
- 发现版本：v0.1.1、v0.1.2（两端均受影响）
- 修复版本：v0.1.3
- 类型：bugfix

## 现象

家里 macOS 安装了 v0.1.1（About 显示 `Version 0.1.1 (0.1.1)`）。
手动 `curl https://api.github.com/repos/X-85/Elwright/releases/latest` 确认 `tag_name = "v0.1.2"`。
点击桌面壳「检查更新」按钮，下方文案显示「已是最新版本（v0.1.1）」——应当是「发现新版本 v0.1.2」。

## 根因

`src-tauri/src/main.rs` 中 `UpdateInfo` 结构体有 snake_case 字段：

```rust
#[derive(Serialize)]
struct UpdateInfo {
    current: String,
    latest: String,
    update_available: bool,
    release_url: String,
}
```

**Tauri 2 的 IPC 返回值序列化不会自动做 snake_case → camelCase 转换**。Rust 端的 `update_available: true` 经 IPC 到达 webview 时还是 `update_available`，但 `src/lib/bridge.ts:38` 的 TypeScript 接口和 `:294` 的调用方读的是 `info.updateAvailable`——永远是 `undefined` → `if (info.updateAvailable)` 走 false 分支 → 显示「已是最新」。

代码库里其他 IPC 命令（如 `get_llm_config`）已经知道这个 Tauri 行为，所以在 bridge.ts 里手动做 `raw.update_available → updateAvailable` 映射；唯独 `check_update` 漏了这一步。

### 验证过程

1. 单元测试：`core::version::tests::detects_patch_minor_major` 等全部通过，`is_newer("v0.1.2", "0.1.1") == true` 行为正确。
2. 独立 Rust 程序模拟 check_update：完整跑一遍 reqwest + serde，输出 `{"update_available":true,...}`——Rust 端没问题。
3. 对照 Tauri 2 源码（`crates/tauri/src/ipc/mod.rs`）：`IpcResponse::body` 直接 `serde_json::to_string(&self)`，**返回路径不做任何 case 转换**；仅参数方向（JS → Rust）默认 camelCase → snake_case。
4. 串起来：Rust 返回 `update_available: true` → 前端读 `info.updateAvailable = undefined` → 显示「已是最新」。

## 修复

`src-tauri/src/main.rs`：给 `UpdateInfo` 加 `#[serde(rename_all = "camelCase")]`：

```rust
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateInfo {
    current: String,
    latest: String,
    update_available: bool,
    release_url: String,
}
```

这样序列化后字段是 `updateAvailable` / `releaseUrl`，前端 `bridge.ts` 和 `App.vue` 不用改。

### 为什么不在 bridge.ts 里手动映射

1. 改动 Rust 一行 vs. 改动 bridge.ts 多行 + 类型断言，前者更小更安全。
2. IPC 命令返回值用 camelCase 已是项目惯例（`Capability.id/name/type` 等都是显式 `#[serde(rename = ...)]`），统一在 Rust 侧解决一致性更好。
3. 留下回归测试在 Rust 端，将来若有人误删 `rename_all` 会立即被单测抓到。

## 回归测试

`src-tauri/src/main.rs::tests::update_info_serializes_camel_case`：
用 `serde_json::to_string` + `from_str` 模拟 IPC 在 webview 里 `JSON.parse` 的路径，断言：
- 序列化输出**必须**包含 `updateAvailable` 和 `releaseUrl`
- **不能**出现 `update_available` / `release_url`

已验证：去掉 `rename_all` 后此测试**确实会失败**（`updateAvailable` 取到 `Null`），加回去后通过。
