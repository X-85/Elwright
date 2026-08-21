#!/usr/bin/env python3
"""Excel(.xlsx) 转 Markdown 表格（离线可用，纯标准库）。

用法:
    python3 xlsx_to_md.py <文件.xlsx> [工作表名或序号，默认第一个]

- 直接解析 xlsx 内部 XML（zipfile + ElementTree），零第三方依赖。
- 单元格取写入时的缓存值：文本/数字正常；公式显示计算结果缓存。
- 不支持: 合并单元格展开、图表、样式。空行输出为空表格行。
"""
import re
import sys
import zipfile
import xml.etree.ElementTree as ET

NS = "{http://schemas.openxmlformats.org/spreadsheetml/2006/main}"
NS_REL = "{http://schemas.openxmlformats.org/officeDocument/2006/relationships}"


def die(msg):
    print(f"错误: {msg}", file=sys.stderr)
    sys.exit(1)


def load_shared_strings(zf):
    try:
        root = ET.fromstring(zf.read("xl/sharedStrings.xml"))
    except KeyError:
        return []
    strings = []
    for si in root.findall(f"{NS}si"):
        strings.append("".join(t.text or "" for t in si.iter(f"{NS}t")))
    return strings


def pick_sheet(zf, selector):
    """按名称或序号选 sheet，返回 (sheet_path, sheet_name)。"""
    workbook = ET.fromstring(zf.read("xl/workbook.xml"))
    rels = ET.fromstring(zf.read("xl/_rels/workbook.xml.rels"))
    rel_map = {r.get("Id"): r.get("Target") for r in rels}
    sheets = []
    for sh in workbook.iter(f"{NS}sheet"):
        rid = sh.get(f"{NS_REL}id")
        target = rel_map.get(rid, "")
        if target and not target.startswith("/"):
            target = "xl/" + target
        sheets.append((target, sh.get("name", "")))

    if not sheets:
        die("工作簿中没有工作表")
    if selector is None:
        return sheets[0]
    if selector.isdigit():
        idx = int(selector) - 1
        if 0 <= idx < len(sheets):
            return sheets[idx]
        die(f"工作表序号超出范围（共 {len(sheets)} 个）")
    for target, name in sheets:
        if name == selector:
            return target, name
    die(f"找不到工作表: {selector}（现有: {', '.join(n for _, n in sheets)}）")


def cell_text(c, shared):
    t = c.get("t", "n")
    if t == "s":
        v = c.find(f"{NS}v")
        return shared[int(v.text)] if v is not None and v.text else ""
    if t == "inlineStr":
        is_el = c.find(f"{NS}is")
        if is_el is not None:
            return "".join(x.text or "" for x in is_el.iter(f"{NS}t"))
        return ""
    v = c.find(f"{NS}v")
    return (v.text or "") if v is not None else ""


def col_index(ref):
    """'BC12' -> 列序号（0 起）。"""
    letters = re.match(r"([A-Z]+)", ref or "")
    if not letters:
        return 0
    n = 0
    for ch in letters.group(1):
        n = n * 26 + (ord(ch) - ord("A") + 1)
    return n - 1


def sheet_rows(zf, path, shared):
    root = ET.fromstring(zf.read(path))
    for row in root.iter(f"{NS}row"):
        cells = {}
        for c in row.findall(f"{NS}c"):
            cells[col_index(c.get("r", ""))] = cell_text(c, shared)
        if not cells:
            yield []
            continue
        width = max(cells) + 1
        yield [cells.get(i, "") for i in range(width)]


def esc(text):
    return text.replace("|", "\\|").replace("\n", " ")


def main():
    if len(sys.argv) < 2:
        die("用法: xlsx_to_md.py <文件.xlsx> [工作表名或序号]")
    path = sys.argv[1]
    selector = sys.argv[2] if len(sys.argv) > 2 else None
    try:
        zf = zipfile.ZipFile(path)
    except OSError as e:
        die(f"无法打开 {path}: {e}")
    except zipfile.BadZipFile:
        die(f"{path} 不是有效的 .xlsx（zip）文件——旧版 .xls 不支持")

    shared = load_shared_strings(zf)
    sheet_path, sheet_name = pick_sheet(zf, selector)
    rows = list(sheet_rows(zf, sheet_path, shared))
    if not rows:
        die(f"工作表「{sheet_name}」为空")

    print(f"<!-- 工作表: {sheet_name} -->")
    header, *body = rows
    print("| " + " | ".join(esc(x) for x in header) + " |")
    print("|" + "---|" * len(header))
    for row in body:
        if len(row) < len(header):
            row = row + [""] * (len(header) - len(row))
        print("| " + " | ".join(esc(x) for x in row[: len(header)]) + " |")


if __name__ == "__main__":
    main()
