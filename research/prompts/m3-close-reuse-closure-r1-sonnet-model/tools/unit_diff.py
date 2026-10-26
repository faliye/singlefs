#!/usr/bin/env python3
"""V2：两棵树（现状、切边）上 check_proof 的闭包，按文件数单元，列出切边之后少了多少。用法：unit_diff.py <现状树> <切边树>
闭包经 lib_function_units.py closure 收，单元行取前四列都在的行（via / refused / key 行不数）。"""
import collections
import subprocess
import sys


def units(tree):
    out = subprocess.run([sys.executable, "scripts/gate/lib_function_units.py", "closure", ".", "scripts/gate/verus-proof-check.py::check_proof"],
                         cwd=tree, capture_output=True, text=True).stdout
    counts = collections.Counter()
    for line in out.splitlines():
        fields = line.split("\t")
        if fields[0] in ("via", "refused", "key") or len(fields) < 4:
            continue
        counts[fields[2].rsplit(":", 1)[0]] += 1
    return counts


before, after = units(sys.argv[1]), units(sys.argv[2])
print(f"单元合计 现状 {sum(before.values())}、切边之后 {sum(after.values())}")
for path in sorted(set(before) | set(after)):
    if before[path] != after[path]:
        print(f"{path}: 现状 {before[path]}、切边之后 {after[path]}、少 {before[path] - after[path]}")
