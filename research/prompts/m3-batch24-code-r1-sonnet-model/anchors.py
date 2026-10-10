#!/usr/bin/env python3
"""第 24 批变异行的锚点核对：用法 anchors.py <副本根>。
 A. mutations.tsv 里变异名以「第24批」或「第 24 批」起头的行（K 21 + G 6 = 27）：六段、原文在源文件里恰好命中一次、替换文与原文不同、
    必须红的测试名在点名的测试文件（或 bin / lib 源）里有 `fn <名>`。
 B. 被删的那一行（「第23批 X E142 导出：建盘之前试分配…」）不在表里；表里没有行的原文含 `try_reserve_exact(free_map_bytes_per_device)`。
 C. 全表每一行：六段，原文在源文件里恰好命中一次（门禁 59 号的锚点腐化判据）；报总行数与命中不是恰好一次的行。"""
import os
import re
import sys

root = sys.argv[1]
rows = []
for line in open(os.path.join(root, "crates/mutations.tsv"), encoding="utf-8").read().split("\n"):
    if line.startswith("#") or not line.strip():
        continue
    rows.append(line)
print(f"TABLE_ROWS {len(rows)}")


def check(row):
    fields = row.split("\t")
    if len(fields) != 6:
        return None, f"fields={len(fields)}"
    name, source, find, replacement, arguments, test_name = fields
    path = os.path.join(root, source)
    if not os.path.exists(path):
        return name, "source-missing"
    text = open(path, encoding="utf-8").read()
    hits = text.count(find.replace("\\n", "\n"))
    return name, hits, find != replacement, arguments, test_name


mine = [r for r in rows if r.split("\t")[0].startswith(("第24批", "第 24 批"))]
print(f"A_ROWS {len(mine)} K={sum(1 for r in mine if r.startswith('第24批 K'))} G={sum(1 for r in mine if r.startswith('第 24 批 GPU'))}")
ok = 0
for row in mine:
    fields = row.split("\t")
    name, source, find, replacement, arguments, test_name = fields
    text = open(os.path.join(root, source), encoding="utf-8").read()
    hits = text.count(find.replace("\\n", "\n"))
    test_file = re.search(r"--test (\S+)", arguments)
    if test_file:
        candidates = [
            os.path.join(d, n)
            for d, _, ns in os.walk(os.path.join(root, "crates"))
            for n in ns
            if n == test_file.group(1) + ".rs" and "/tests" in d
        ]
    else:
        bin_name = re.search(r"--bin (\S+)", arguments)
        candidates = [
            os.path.join(d, n)
            for d, _, ns in os.walk(os.path.join(root, "crates"))
            for n in ns
            if bin_name and n == bin_name.group(1) + ".rs"
        ]
    has_test = any(("fn " + test_name) in open(p, encoding="utf-8").read() for p in candidates)
    good = hits == 1 and find != replacement and has_test
    ok += good
    print(f"A {'OK ' if good else 'BAD'} hits={hits} differs={find != replacement} has_test={has_test} {name[:46]}")
print(f"A_OK {ok}/{len(mine)}")

deleted = "第23批 X E142 导出：建盘之前试分配空闲图只要 0 字节"
print("B_DELETED_ROW_PRESENT", any(r.startswith(deleted) for r in rows))
print("B_ROWS_WITH_TRY_RESERVE_ANCHOR", sum(1 for r in rows if "try_reserve_exact(free_map_bytes_per_device)" in r.split("\t")[2:3][0:1].__str__()))

bad = []
for row in rows:
    result = check(row)
    if result[0] is None or result[1] == "source-missing":
        bad.append((row.split("\t")[0][:60], result[1]))
    elif result[1] != 1:
        bad.append((result[0][:60], f"hits={result[1]}"))
print(f"C_ROWS {len(rows)} C_NOT_EXACTLY_ONE {len(bad)}")
for item in bad:
    print("C_BAD", item)
