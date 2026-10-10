#!/usr/bin/env python3
"""apply-row.py <仓副本根> <mutations.tsv 行号>：把变异表那一行（名、文件、原文、替换文、cargo 参数、点名的用例）施加到副本上，
原文恰好命中一次才改（否则退 3）；把 cargo 参数与点名用例打到标准输出（两行）。只改副本。"""
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
line_number = int(sys.argv[2])
row = (root / "crates/mutations.tsv").read_text().split("\n")[line_number - 1].split("\t")
name, rel, old, new, args, test = row[:6]
old = old.replace("\\n", "\n").replace("\\t", "\t")
new = new.replace("\\n", "\n").replace("\\t", "\t")
path = root / rel
text = path.read_text()
if text.count(old) != 1:
    print(f"{name}: 原文命中 {text.count(old)} 次", file=sys.stderr)
    sys.exit(3)
path.write_text(text.replace(old, new))
print(args)
print(test)
