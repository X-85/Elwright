# 验证记录（2026-08-21，家里 macOS，node 24.15 / vite 8.2）

## 构建

- `npm install vue marked && npm install -D vite @vitejs/plugin-vue typescript` 成功。
- `npm run build`（vite build）成功：18 modules，产物 111 KB（gzip 40 KB）。

## Dev API 冒烟（curl）

| 检查 | 结果 |
|---|---|
| `GET /api/capabilities` | ✅ 返回 24 项能力 |
| `GET /api/file?path=resources/docs/tech-grill-sop.md` | ✅ 返回文件全文 |
| `GET /api/file?path=capabilities.json`（目录穿越防护） | ✅ 403「仅允许访问 resources/ 下的文件」 |
| `GET /`（页面） | ✅ HTTP 200 |

## 浏览器 UI 冒烟（In-app Browser，DOM 快照逐项验证）

| 场景 | 结果 |
|---|---|
| 首屏加载 | ✅ 侧栏（品牌/筛选/搜索/计数）+ 24 项列表（名称/类型徽标/id/分类/⚡离网标记） |
| 「技能型」筛选 | ✅ 列表变为 6/24，按钮 active 态 |
| 技能详情（八层拷问） | ✅ 模板、降级提示（含 SOP 路径）、附加输入框、调用按钮 |
| 点击「调用」 | ✅ 展示「【预览模式】未接 LLM」横幅 + tech-grill-sop.md 完整 Markdown 渲染（标题/引用/列表/加粗均正确） |
| 知识型（公司家里项目共建指南） | ✅ 选中即渲染 doc 字段文档（标题、正文、结构均正常） |
| 脚本型（文档关键字搜索） | ✅ 入口路径、参数输入、运行按钮；点击后展示「预览模式无法执行脚本」说明与等价 CLI 命令 |

## 未验证项（说明）

- Tauri IPC 路径：适配器接口已预留（`createBridge()`），Rust 侧命令与 `tauri build` 属阶段 3b 任务。
- 真实脚本执行 / 真实 LLM 调用在 UI 中的表现：需 Tauri 桌面壳（浏览器预览按设计降级为说明文案）。

## 结论

前端构建、API 读取、四类 UI 路径（列表/筛选/知识渲染/技能降级/脚本说明）全部通过。**阶段 3 前端先行部分完成，可进入用户确认环节**；后续任务：阶段 3b（Tauri IPC + 打包）。
