#!/usr/bin/env python3
"""变异表第 16 行与第 3102–3109 行：原文在冻结副本里各命中几次、第一处在第几行。用法：SINGLEFS_FROZEN_TREE=<冻结副本> python3 anchors.py（输出行以 ANCHOR 开头）"""
import os
tree = os.environ["SINGLEFS_FROZEN_TREE"]
lines = open(os.path.join(tree, "crates/mutations.tsv"), encoding="utf-8").read().split("\n")
for number in [16] + list(range(3102, 3110)):
    fields = lines[number - 1].split("\t")
    assert len(fields) == 6, (number, len(fields))
    path, original = fields[1], fields[2].replace("\\n", "\n")
    source = open(os.path.join(tree, path), encoding="utf-8").read()
    index = source.find(original)
    line = source[:index].count("\n") + 1 if index >= 0 else None
    print(f"ANCHOR row={number} hits={source.count(original)} line={line} file={path}")
