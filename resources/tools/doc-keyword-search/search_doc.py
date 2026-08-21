#!/usr/bin/env python3
"""文档关键字搜索（离线可用，纯标准库）。

用法:
    python3 search_doc.py <关键字> [路径 ...]

- 路径省略时搜索当前目录；可给多个文件或目录。
- 递归搜索 .md / .txt / .log 文本文件，忽略隐藏目录与 node_modules/target。
- 大小写不敏感；输出 `文件:行号: 内容`，超长行截断到 200 字符。
"""
import os
import sys

TEXT_EXTS = {".md", ".txt", ".log"}
SKIP_DIRS = {".git", "node_modules", "target", "__pycache__", ".venv", "dist"}
MAX_LINE = 200


def iter_files(paths):
    for p in paths:
        if os.path.isfile(p):
            yield p
        elif os.path.isdir(p):
            for base, dirs, files in os.walk(p):
                dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
                for name in files:
                    if os.path.splitext(name)[1].lower() in TEXT_EXTS:
                        yield os.path.join(base, name)


def search(keyword, paths):
    hits = 0
    files = 0
    needle = keyword.lower()
    for path in iter_files(paths):
        files += 1
        try:
            with open(path, "r", encoding="utf-8", errors="replace") as fh:
                for lineno, line in enumerate(fh, 1):
                    if needle in line.lower():
                        text = line.rstrip("\n").strip()
                        if len(text) > MAX_LINE:
                            text = text[:MAX_LINE] + "…"
                        print(f"{path}:{lineno}: {text}")
                        hits += 1
        except OSError as e:
            print(f"跳过不可读文件 {path}: {e}", file=sys.stderr)
    print(f"\n共搜索 {files} 个文件，命中 {hits} 处", file=sys.stderr)


def main():
    if len(sys.argv) < 2 or not sys.argv[1].strip():
        print("用法: search_doc.py <关键字> [路径 ...]", file=sys.stderr)
        sys.exit(1)
    keyword = sys.argv[1]
    paths = sys.argv[2:] or ["."]
    if not any(os.path.exists(p) for p in paths):
        print(f"错误: 路径不存在: {', '.join(paths)}", file=sys.stderr)
        sys.exit(1)
    search(keyword, paths)


if __name__ == "__main__":
    main()
