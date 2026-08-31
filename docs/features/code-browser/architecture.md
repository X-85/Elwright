# 架构（Architecture）

```text
App.vue
  └─ CodeBrowserView.vue
       ├─ ProjectPicker
       ├─ ProjectTree
       ├─ CodeTabs
       ├─ CodeContextActions
       └─ LightweightSymbolIndex
            └─ Bridge
                 ├─ desktop: Tauri IPC
                 └─ preview: 明确受限的只读适配器
```

## 边界

- 前端组件只依赖 `src/lib/bridge.ts` 的 `Bridge` 接口，不直接调用 Tauri API 或 `fetch`。
- 桌面端通过 Tauri IPC 实现目录选择、目录树读取、文件读取和项目内搜索。
- 读取接口必须限制在用户选择的项目根目录内，拒绝路径穿越和项目根外访问。
- 大文件、二进制文件和超深目录需要大小或深度限制，避免一次加载阻塞 UI。
- 代码查看不依赖 Rust core 的能力执行链；文件访问属于桌面壳能力，后续再决定是否抽取共享模块。
- 轻量跳转采用按需扫描和缓存的项目内符号索引，首批只识别 Java 类型、`implements`、`extends` 和常见方法声明；索引不常驻占用大量内存，项目切换或文件变化时按需失效。
- 轻量索引无法确认唯一目标时返回候选或搜索结果；不为了“看起来能跳”而猜测目标。
- Java Language Server 属于后续可选增强，不纳入第一阶段默认运行时依赖。
- 代码发送到 AI 时沿用现有 AI 对话入口和用户确认边界，不在代码浏览器中另建模型客户端。

## 第一阶段建议数据模型

- `Project`: `id`、`name`、`rootPath`、`lastOpenedAt`。
- `TreeEntry`: 相对路径、名称、文件/目录类型、大小和可读状态。
- `CodeDocument`: 项目 id、相对路径、内容、语言、读取时间。
- `CodeSelection`: 项目 id、相对路径、起止行列和用户确认后的内容。
- `SymbolLocation`: 符号名称、种类、相对路径、行号和关系类型。

项目和最近文件元数据写入用户配置层；代码内容只在当前页面状态中保留，除非用户显式保存工作记录。

## 阶段④补丁链路（受控写入）

```text
ChatView.vue (助手消息内识别 ```diff 围栏)
  └─ PatchPreviewDialog.vue (三栏预览 + 勾选拒绝 + 确认)
       └─ Bridge.applyPatchPreview / applyPatchApply / applyPatchRevert / listPatchSnapshots
            └─ Tauri IPC → core::patch
                 ├─ parse_unified_diff: 多文件 + 多 hunk 解析，识别 --- / +++ / @@ 头
                 ├─ apply_hunks_to_content: 按 hunk header(oldStart, oldCount) 切片，
                 │    比对上下文行，错位/上下文不匹配返回 Err，调用方把它降到 warnings 并跳过
                 ├─ is_sensitive_path: 黑名单 .env / .pem / .key / .ssh / .aws /
                 │    node_modules / target / .git 任一命中直接拒绝
                 ├─ sha256_hex: 用 sha2 计算当前内容指纹，存进 snapshot 便于回滚核对
                 └─ snapshot_path: ~/.elwright/code-browser/applied-patches.json
```

- 前端 `src/lib/patch.ts` 的 `extractFirstDiff` 只负责识别 `\`\`\`diff` 围栏（粗筛，无路径校验 / 上下文校验），权威校验在后端 `core::patch`。
- 预览阶段不写盘，`applyPatchApply` 才会触发写入；写入前把每个文件的原内容 + SHA-256 入快照。
- 快照文件用 atomic write（写 tmp 再 rename），避免崩溃中间态。
- 回滚按快照 id 找到 `preImages`，逐文件覆盖写回并核对 SHA-256。
- 解析与上下文比对都在后端，前端只负责「勾选拒绝 / 三栏渲染」。
