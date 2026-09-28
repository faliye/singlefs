#!/usr/bin/env python3
"""按 crates/mutations.tsv 里一行（按变异名认）改坏仓副本里那个文件：原文必须恰好命中一次（同门禁 59 号的锚点规矩）。
只在草稿里的仓副本上用。用法：apply_mutation_row.py <仓副本根> <变异名>"""
import os
import sys

root, name = sys.argv[1], sys.argv[2]
rows = [line.rstrip("\n").split("\t") for line in open(os.path.join(root, "crates/mutations.tsv"), encoding="utf-8") if not line.startswith("#")]
matching = [row for row in rows if row[0] == name]
assert len(matching) == 1, f"变异名「{name}」在 mutations.tsv 里命中 {len(matching)} 行"
_, relative, original, replacement = matching[0][:4]
original, replacement = original.replace("\\n", "\n"), replacement.replace("\\n", "\n")
path = os.path.join(root, relative)
text = open(path, encoding="utf-8").read()
assert text.count(original) == 1, f"{relative} 里原文命中 {text.count(original)} 次"
with open(path + ".mutated", "x", encoding="utf-8") as handle:
    handle.write(text.replace(original, replacement))
os.replace(path + ".mutated", path)
print(f"mutated {relative} by row「{name}」")
