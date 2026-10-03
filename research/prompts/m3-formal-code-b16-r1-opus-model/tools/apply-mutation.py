#!/usr/bin/env python3
"""把 crates/mutations.tsv 第 N 行（文件行号）的变异施加到副本树 <root> 上：原文恰好命中一次才改，改完回读确认。
用法：apply-mutation.py <副本根> <行号>    （副本根下有 crates/mutations.tsv 与被改的源文件）"""
import sys
root, line_number = sys.argv[1], int(sys.argv[2])
rows = open(f"{root}/crates/mutations.tsv", encoding="utf-8").read().split("\n")
fields = rows[line_number - 1].split("\t")
name, path, original, replacement = fields[0], fields[1], fields[2], fields[3]
original = original.replace("\\n", "\n")
replacement = replacement.replace("\\n", "\n")
source_path = f"{root}/{path}"
source = open(source_path, encoding="utf-8").read()
hits = source.count(original)
if hits != 1:
    sys.exit(f"原文命中 {hits} 次（要恰好 1 次）：{name}")
mutated = source.replace(original, replacement)
with open(source_path + ".mutating", "w", encoding="utf-8") as out:
    out.write(mutated)
import os
os.replace(source_path + ".mutating", source_path)
if open(source_path, encoding="utf-8").read() != mutated:
    sys.exit("回读对不上")
print(f"applied line {line_number}: {name[:80]} -> {path}")
