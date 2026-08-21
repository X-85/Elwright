#!/usr/bin/env python3
"""Word(.docx) 转 Markdown（离线可用，纯标准库）。

用法:
    python3 docx_to_md.py <文件.docx>

- 直接解析 docx 内部 XML（zipfile + ElementTree），零第三方依赖。
- 支持: 标题(Heading 1-6 → #)、正文段落、表格（含表头行）。
- 不支持: 行内加粗/斜体/图片/列表编号样式（列表按普通段落输出）。
"""
import sys
import zipfile
import xml.etree.ElementTree as ET

W = "{http://schemas.openxmlformats.org/wordprocessingml/2006/main}"


def die(msg):
    print(f"错误: {msg}", file=sys.stderr)
    sys.exit(1)


def para_text(p):
    return "".join(t.text or "" for t in p.iter(f"{W}t"))


def para_md(p):
    text = para_text(p).strip()
    if not text:
        return ""
    style = ""
    ppr = p.find(f"{W}pPr")
    if ppr is not None:
        ps = ppr.find(f"{W}pStyle")
        if ps is not None:
            style = ps.get(f"{W}val", "")
    levels = {"Heading1": 1, "Heading2": 2, "Heading3": 3, "Heading4": 4, "Heading5": 5, "Heading6": 6}
    if style in levels:
        return "#" * levels[style] + " " + text
    if style in ("Quote", "IntenseQuote"):
        return "> " + text
    return text


def table_md(tbl):
    lines = []
    rows = tbl.findall(f"{W}tr")
    for i, tr in enumerate(rows):
        cells = ["".join(t.text or "" for t in tc.iter(f"{W}t")).strip()
                 for tc in tr.findall(f"{W}tc")]
        cells = [c.replace("|", "\\|") for c in cells] or [""]
        lines.append("| " + " | ".join(cells) + " |")
        if i == 0:
            lines.append("|" + "---|" * len(cells))
    return "\n".join(lines)


def main():
    if len(sys.argv) < 2:
        die("用法: docx_to_md.py <文件.docx>")
    path = sys.argv[1]
    try:
        zf = zipfile.ZipFile(path)
        root = ET.fromstring(zf.read("word/document.xml"))
    except OSError as e:
        die(f"无法打开 {path}: {e}")
    except (zipfile.BadZipFile, KeyError):
        die(f"{path} 不是有效的 .docx 文件——旧版 .doc 不支持")
    except ET.ParseError:
        die(f"{path} 内部 XML 损坏")

    body = root.find(f"{W}body")
    if body is None:
        die("文档没有正文内容")
    out = []
    for el in body:
        if el.tag == f"{W}p":
            line = para_md(el)
            if line:
                out.append(line)
        elif el.tag == f"{W}tbl":
            out.append(table_md(el))
    print("\n\n".join(out))


if __name__ == "__main__":
    main()
