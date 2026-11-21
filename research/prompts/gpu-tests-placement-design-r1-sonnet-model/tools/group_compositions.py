#!/usr/bin/env python3
"""读验证包构建脚本写进 OUT_DIR 的判器键表（judging_units_keys.tsv），逐组数组成清单里属于点名模块的单元：
限定名以 `singlefs_verification::<模块名>` 起头的（含 lib.rs 里 `pub mod <模块名>;` 那一个声明单元）。
用法：group_compositions.py <键表> <模块名>…（模块名写 sha256 这一类，不带 .rs）"""
import collections
import sys

table, modules = sys.argv[1], sys.argv[2:]
groups = collections.OrderedDict()
current = None
for line in open(table, encoding="utf-8"):
    parts = line.rstrip("\n").split("\t")
    if parts[0] == "group":
        current = parts[1]
        groups[current] = []
    elif parts[0] == "unit" and current is not None and len(parts) > 4:
        groups[current].append(parts[3])
print(f"name=groups count={len(groups)}")
for group, names in groups.items():
    counted = collections.Counter()
    for qualified in names:
        for module in modules:
            prefix = "singlefs_verification::" + module
            if qualified == prefix or qualified.startswith(prefix + "::"):
                counted[module] += 1
    if counted:
        print(f"group={group} units={len(names)} in_moved_modules={dict(counted)}")
