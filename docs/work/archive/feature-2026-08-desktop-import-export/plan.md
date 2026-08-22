# 桌面端能力导入/导出/删除（用户叠加层）

## 背景与目标

装机后 bundle 根只读或更新即清零（macOS .app 整体替换 / Windows Program Files 需管理员），照搬 CLI「写回 reg.root」的导入在桌面端是假功能。本任务引入**用户叠加层 overlay**：

```
~/.elwright/
├── config.json       # 已有：LLM 配置
├── capabilities.json # 新增：用户自定义条目
└── resources/        # 新增：用户自定义脚本/文档
```

- 合并加载：bundle 注册表 + 用户注册表，同 id 用户条目**遮蔽**内置条目。
- 文件解析双根：先 `~/.elwright/resources/`，再回退 bundle `resources/`。
- app 更新只动 bundle，用户能力不丢；Windows 无权限问题；「内置/自定义」天然可区分。
- CLI 与桌面壳看到同一合并视图。

## v1 范围（已拍板）

- 单项导出（与 CLI `elwright-skill/0.1` 格式一致）；批量导出留 v2。
- 删除自定义项（含其 overlay 文件）；内置项不可删；v1 不做「隐藏内置」。
- 浏览器预览：导出走 Blob 下载；导入/run/invoke 维持降级文案。

## 改动面

| 位置 | 改动 |
|---|---|
| `core/registry.rs` | `load_merged`：base + overlay 合并、origin 标记；`resolve_entry`/文件读取走双根；`user_root()` 定位 `~/.elwright`（`ELWRIGHT_USER_ROOT` env 可覆盖，便于测试） |
| `core/export.rs` | `import_capability` 目标根参数化（导入总写 overlay）；新增 `delete_capability`（仅 overlay 条目，删条目+引用文件） |
| `src-tauri/src/main.rs` | `import_capability` / `export_capability` / `delete_capability` 三个 IPC + `tauri-plugin-dialog`；IPC 返回 capability 带 origin |
| `src/lib/bridge.ts` | Bridge 加 `importCapability/exportCapability/deleteCapability`；Capability 加 `origin: 'builtin'|'custom'`；浏览器导出=注册表数据组 bundle→Blob 下载 |
| `src/App.vue` | 侧栏 [导入…] 按钮、详情页 [导出][删除]（仅自定义）、列表「自定义」徽标、id 冲突确认框（=--force） |

## 安全与边界

- overlay 导入沿用 `safe_relative`（resources/ 前缀、拒 `..`/绝对路径）。
- 遮蔽只影响展示与调用解析，不修改 bundle 文件。
- core 不引 tauri 依赖（AGENTS.md 边界）。

## 验证

- 单测：合并/遮蔽/origin；overlay 双根文件解析；删除（含文件清理、内置拒删）；导入写 overlay 不碰 base。
- `cargo test` + `npm run build` + 浏览器预览（徽标/导出下载/导入降级文案）+ Tauri debug 构建冒烟。
