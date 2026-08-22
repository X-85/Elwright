#!/usr/bin/env python3
"""文本统计：统计文件的行数、字符数、中文字符数、英文单词数。

用法：python3 text_stats.py <文件路径>
纯标准库，零第三方依赖（离网底线）。
"""

import sys
from pathlib import Path


def count_text(text: str) -> dict:
    chinese = sum(1 for ch in text if "\u4e00" <= ch <= "\u9fff")
    # 英文单词：连续的英文字母序列
    words = []
    current = []
    for ch in text:
        if ch.isascii() and ch.isalpha():
            current.append(ch)
        elif current:
            words.append("".join(current))
            current = []
    if current:
        words.append("".join(current))
    return {
        "lines": text.count("\n") + (0 if text.endswith("\n") or not text else 1),
        "chars": len(text),
        "chinese": chinese,
        "words": len(words),
    }


def main() -> int:
    if len(sys.argv) != 2:
        print("错误：请提供一个文件路径。用法：text_stats.py <文件路径>", file=sys.stderr)
        return 1
    path = Path(sys.argv[1])
    if not path.is_file():
        print(f"错误：文件不存在或不是普通文件：{path}", file=sys.stderr)
        return 1
    try:
        text = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        print(f"错误：文件不是 UTF-8 编码：{path}", file=sys.stderr)
        return 1

    stats = count_text(text)
    print(f"文件：{path}")
    print(f"行数：{stats['lines']}")
    print(f"字符数：{stats['chars']}")
    print(f"中文字符：{stats['chinese']}")
    print(f"英文单词：{stats['words']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
