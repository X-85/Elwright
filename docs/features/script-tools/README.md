# 脚本型能力（第一批：3 个通用工具）

## 功能简介

`resources/tools/` 下的离线脚本能力，经 `ew run <id>`（CLI）或桌面壳「运行」按钮（IPC 同一 executor）调用。本批 3 个均为纯 Python 标准库实现，零第三方依赖——装了 python3 即可用，离网可跑。

## 当前状态

第一批完成（2026-08-21）。其余 10 个 entry 待公司机器原版脚本导入。

## 能力清单

| id | 入口 | 功能 | 限制 |
|---|---|---|---|
| `doc-keyword-search` | `search_doc.py` | 递归搜 .md/.txt/.log 中的关键字（大小写不敏感，`文件:行号: 内容`） | 跳过 .git/node_modules/target 等目录；行截断 200 字符 |
| `xlsx-to-md` | `xlsx_to_md.py` | .xlsx → Markdown 表格，支持按名称/序号选工作表 | 纯 stdlib 解析；公式取缓存值；不支持合并单元格展开/样式 |
| `docx-to-md` | `docx_to_md.py` | .docx → Markdown（Heading 1-6 → #，段落，表格） | 不支持行内格式/图片；旧版 .xls/.doc 不支持（提示友好报错） |

## 已知限制（跨平台）

executor 固定用 `python3` 解释器——Linux/macOS 默认有；Windows 需 python3 别名（或后续让 executor 探测 `python`/`py`，列为后续改进）。

## 相关文档

- 任务记录：`docs/work/active/feature-2026-08-stage5-script-tools/`
