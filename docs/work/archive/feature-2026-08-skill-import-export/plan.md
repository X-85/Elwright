# 能力导入/导出（skill portability）

## 目标

把一项能力（注册表条目 + 其引用的脚本/文档/SOP 文件）打包成单文件，跨机器/跨用户分享——兑现「开源普惠」的分享场景。

## 设计

- 导出格式 `*.elw.json`：纯 JSON，零新依赖：
  `{schema: "elwright-skill/0.1", capability: {原条目原样}, files: [{path, content}]}`
  content 为 UTF-8 文本直存（脚本/文档皆文本；二进制暂不支持，报友好错误）。
- 文件清单自动收集：`entry` / `doc` / `degradeDoc` 引用且真实存在的文件。
- 导入校验：schema 版本；路径必须为 `resources/` 前缀的相对路径（拒 `..`/绝对路径——防路径逃逸）；id 冲突默认报错，`--force` 覆盖。
- 写回 capabilities.json：serde_json 开 `preserve_order` 特性，保持既有键序不重排。

## CLI

- `ew export <id> [文件名]`（缺省打印到 stdout）
- `ew import <文件> [--force]`

## 非目标（后续）

- 桌面壳导入/导出 UI（需 tauri 文件对话框插件，独立任务）
- 二进制资源打包（zip/base64）

## 验证

- 单测：临时根 A 导出 → 临时根 B 导入，注册表与文件完整还原；路径逃逸样本被拒。
- 手动：真实仓库 round-trip（export tech-grill → import 到副本根）。
