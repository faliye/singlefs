#!/usr/bin/env python3
"""第 23 批变异行的锚点核对：用法 anchors.py <副本根>。
行集合：mutations.tsv 里变异名含「第 23 批」「第23批」的行，加三行改名连带的与那一行 SCRATCH_WORDS 的替换行。
每行核：六段、原文在源文件里恰好命中一次、替换文与原文不同、必须红的测试名在点名的测试文件（或 lib / bin 源）里有 `fn <名>`。"""
import os
import re
import sys

root = sys.argv[1]
extra_prefixes = (
    "删除批第 21 批 N3：可写挂载",
    "删除批第 22 批 N5：可写挂载重建",
    "第 5 批 GPU 判器：派活给每个状态的草稿区",
)
rows = []
for line in open(os.path.join(root, "crates/mutations.tsv"), encoding="utf-8").read().split("\n"):
    if line.startswith("#") or not line.strip():
        continue
    name = line.split("\t")[0]
    if "第 23 批" in name or "第23批" in name or name.startswith(extra_prefixes):
        rows.append(line)
ok = 0
for row in rows:
    fields = row.split("\t")
    name, source, find, replacement, arguments, test_name = fields
    text = open(os.path.join(root, source), encoding="utf-8").read()
    hits = text.count(find.replace("\\n", "\n"))
    differs = find != replacement
    package = re.search(r"-p (\S+)", arguments).group(1)
    test_file = re.search(r"--test (\S+)", arguments)
    if test_file:
        candidates = [
            os.path.join(directory, name_)
            for directory, _, names in os.walk(os.path.join(root, "crates"))
            for name_ in names
            if name_ == test_file.group(1) + ".rs" and "/tests" in directory
        ]
        has_test = any(("fn " + test_name) in open(path, encoding="utf-8").read() for path in candidates)
    else:
        has_test = any(
            ("fn " + test_name) in open(os.path.join(directory, name_), encoding="utf-8").read()
            for directory, _, names in os.walk(os.path.join(root, "crates", package.replace("_", "-")))
            for name_ in names
            if name_.endswith(".rs")
        )
    good = len(fields) == 6 and hits == 1 and differs and has_test
    ok += good
    print(f"ANCHOR hits={hits} differs={differs} test_fn_found={has_test} {name[:60]}")
print(f"ANCHOR_RESULT rows={len(rows)} ok={ok}")
