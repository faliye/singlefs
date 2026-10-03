#!/usr/bin/env python3
"""E171 坏实现表的挑法（登记第五节 5.2）：从 crates/mutations.tsv 里挑改实现的行。
用法：select_bad_implementations.py <mutations.tsv> <步长k> [--first-slice]
规则：文件列以 crates/singlefs-core/src/ 起头、或等于 crates/singlefs-replay-prefix/src/lib.rs 的行（注释行与非六段的行不算）；
按文件名字典序分组，组内保持表里的先后；一组 n 行取 m = ceil(n/k) 行，下标 floor(i*n/m)（i = 0..m-1，0 起）。
--first-slice 只打 i = 0 那一行（每个文件一行）。输出：文件<TAB>组内下标<TAB>i<TAB>变异名。
"""
import sys
from collections import OrderedDict

path, k = sys.argv[1], int(sys.argv[2])
first_slice = "--first-slice" in sys.argv[3:]
groups = {}
with open(path, encoding="utf-8") as f:
    for line in f:
        line = line.rstrip("\n")
        if line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) != 6:
            continue
        name, file = fields[0], fields[1]
        if file.startswith("crates/singlefs-core/src/") or file == "crates/singlefs-replay-prefix/src/lib.rs":
            groups.setdefault(file, []).append(name)
for file in sorted(groups):
    names = groups[file]
    n = len(names)
    m = -(-n // k)
    for i in range(m):
        if first_slice and i != 0:
            continue
        j = (i * n) // m
        print(f"{file}\t{j}\t{i}\t{names[j]}")
