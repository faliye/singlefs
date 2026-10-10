#!/usr/bin/env python3
"""把一段原型代码插到文件最后一个 `}` 之前（文件末尾那个 `mod tests` 的收尾）。用法：inject.py <文件> <片段>。"""
import sys
from pathlib import Path
path, snippet = Path(sys.argv[1]), Path(sys.argv[2]).read_text()
text = path.read_text().rstrip("\n")
assert text.endswith("}"), "文件不以 } 收尾"
path.write_text(text[:-1] + snippet + "}\n")
print(f"injected {sys.argv[2]} into {sys.argv[1]}")
