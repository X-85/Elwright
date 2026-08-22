# 验证记录（2026-08-21，家里 macOS，python3 3.x）

样本：程序构造的最小合法 .xlsx（sharedStrings/数字/公式缓存值/竖线单元格）与 .docx（Heading1-2/中文段落/两列表格），结构与 Office 写出一致。

## 端到端（ew run，真实 executor 链路）

| 场景 | 结果 |
|---|---|
| `ew run xlsx-to-md sample.xlsx` | ✅ 工作表名注释 + 表头/分隔行 + 数据行；中文、数字、公式缓存值（`=SUM` 显 2）、竖线转义 `\|` 全正确 |
| `ew run xlsx-to-md sample.xlsx 产品清单` | ✅ 按工作表名选择 |
| `ew run docx-to-md sample.docx` | ✅ `#`/`##` 标题、段落、Markdown 表格（含表头分隔）正确 |
| `ew run doc-keyword-search 离线 resources/docs` | ✅ 命中多文件多行，`文件:行号: 内容` 格式 |
| `ew run doc-keyword-search 离网 resources/docs` | ✅ 0 命中（文档用词为「离线」，语义正确）+ 汇总「共搜索 9 个文件」 |

## 错误路径（全部友好中文报错，不崩溃）

| 场景 | 输出 |
|---|---|
| xlsx 文件不存在 | `错误: 无法打开 /tmp/nope.xlsx: [Errno 2] ...` |
| 把 .xlsx 喂给 docx 脚本 | `错误: ... 不是有效的 .docx 文件——旧版 .doc 不支持` |
| 搜索关键字为空 | `用法: search_doc.py <关键字> [路径 ...]` |
| 缺参数 | `用法: xlsx_to_md.py <文件.xlsx> [工作表名或序号]` |

## 未验证项

- Windows `python3` 别名场景（见 Feature README 已知限制）。
- 真实 Office 保存的复杂文档（多 sheet 大文件、嵌套表格）——stdlib 解析覆盖常用子集，复杂样本待用户实际使用反馈。

## 结论

3 个脚本的正路径、参数变体、错误路径全部通过。CI 待推送后确认。
