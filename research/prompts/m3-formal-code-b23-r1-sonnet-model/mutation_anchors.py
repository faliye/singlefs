#!/usr/bin/env python3
"""第 23 批新增的 7 行变异（crates/mutations.tsv 第 2407–2413 行）的锚点：原文在冻结副本的目标文件里命中几次、替换后与原文是否不同。
用法：mutation_anchors.py <冻结副本 tree 根>。只读，不改任何文件，不编译不跑。
"""
import sys

root = sys.argv[1]
rows = [line.rstrip("\n").split("\t") for line in open(f"{root}/crates/mutations.tsv", encoding="utf-8")]
print(f"mutations.tsv 共 {len(rows)} 行")
for number, row in enumerate(rows, 1):
    if not (2407 <= number <= 2413):
        continue
    name, path, original, replacement = row[0], row[1], row[2].replace("\\n", "\n"), row[3].replace("\\n", "\n")
    source = open(f"{root}/{path}", encoding="utf-8").read()
    print(f"第 {number} 行 命中 {source.count(original)} 次 替换不同={original != replacement} 目标={path} 测试={row[4] if len(row) > 4 else '?'} :: {name[:48]}")
