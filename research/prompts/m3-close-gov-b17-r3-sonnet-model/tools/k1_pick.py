#!/usr/bin/env python3
"""挑一个 crates/singlefs-core/src 里的 fn：在 N 行以上的点名测试闭包里、它上面没有属性。打 文件 / fn 名 / 进了几行。
用法：k1_pick.py <脚本所在仓> <crates 树根>"""
import collections
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import k1_common

script_repository, root = sys.argv[1:3]
module = k1_common.load_rows_module(script_repository)
rows, memory_max, timeout_seconds = k1_common.table_rows(module, root)
rows = [row for row in rows if row.path.startswith("crates/singlefs-core/src/")][:400]
key_context, components, _library = k1_common.key_context_for(module, root, rows, memory_max, timeout_seconds)
count = collections.Counter()
for row in rows:
    comp = components[row.line_number]
    closure = key_context.closures.get(comp.test_closure) if comp.test_closure else None
    if closure is None:
        continue
    for member in closure.members:
        if member.kind == "fn" and member.file.startswith("crates/singlefs-core/src/") and "::" not in member.name:
            count[(member.file, member.name, member.lines)] += 1
picked = []
for (file, name, lines), rows_in in count.most_common():
    with open(os.path.join(root, file), encoding="utf-8") as handle:
        source = handle.read().split("\n")
    first = int(lines.split("-")[0])
    line = source[first - 1]
    if line.lstrip().startswith(("fn ", "pub fn ", "pub(crate) fn ")) and not source[first - 2].lstrip().startswith("#[") and 3 <= rows_in <= 400:
        picked.append((file, name, rows_in, first))
for file, name, rows_in, first in picked[:5]:
    print(f"fn_candidate file={file} name={name} line={first} rows_in_closure={rows_in}")
