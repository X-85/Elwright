# 阶段 5：脚本型能力落地（第一批：3 个通用脚本）

## 背景

阶段 4 完成后路线图已走完，但产品最大缺口：13 个脚本型能力的 entry 全部指向不存在的文件，「离网 70%」承诺名存实亡。本阶段开始逐批补齐。

## 本批范围（家里 Mac 可完成的通用 3 个）

- `doc-keyword-search` → `resources/tools/doc-keyword-search/search_doc.py`：递归搜索文本文件中的关键字（.md/.txt，大小写不敏感，输出 文件:行号:内容）。
- `xlsx-to-md` → `resources/tools/xlsx-to-md/xlsx_to_md.py`：xlsx → Markdown 表格。纯 stdlib（zipfile + xml.etree 解析 sheet/sharedStrings），零第三方依赖。
- `docx-to-md` → `resources/tools/docx-to-md/docx_to_md.py`：docx → Markdown（标题/段落/表格）。纯 stdlib 同法。

约束：脚本必须**无第三方依赖**（离网安装即用）、中文输出、跨平台路径处理、错误信息友好（非技术用户可读）。

## 明确留位（不在本批）

其余 10 个脚本 entry 强绑定公司环境/原 toolbox 脚本（deploy.ps1、kb-tools、vscode-ext 等），等公司机器原版导入，不在 Mac 上凭空重写。

## 风险

- stdlib 解析 xlsx/docx 只覆盖常用子集（不支持的公式重算/合并单元格等需在输出中明确提示）。
- executor 固定用 `python3` 解释器：Windows 上若无 python3 别名会失败——已知限制，记录在 Feature 文档，阶段 6 再议（可改 executor 探测 python/py）。
- 桌面壳 run_script 走同一 executor，无需单独适配。

## 验证方式

- 每个脚本用程序构造的样本（.xlsx/.docx 用 zipfile+XML 手工构造，.md 现成）跑 `ew run <id>` 端到端验证。
- 错误路径：不存在文件、缺参数、空表——报错友好不崩溃。
- 提交推送过 CI（三平台）。
