#!/usr/bin/env python3
"""crates-selection.tsv 里零覆盖（挑出 0 条测试）的变异，按落在 fn 里还是 fn 外（常量初始化式）、按库里这一版的结局拆开。
用法：crates_zero_coverage_split.py <模型目录> <HEAD 导出的副本根> <库目录这一版>
"""
import collections
import json
import os
import sys

model, root, library = sys.argv[1], sys.argv[2], sys.argv[3]
sys.argv = sys.argv[:1] + ["-", "-"]
sys.path.insert(0, model)
import run  # noqa: E402

mutants = {m["sequence"]: m for m in json.load(open(os.path.join(library, "list.json")))}
rows = [line.rstrip("\n").split("\t") for line in open(os.path.join(model, "evidence", "crates-selection.tsv"))][1:]
texts, tables = {}, {}
by_package, by_outcome = collections.Counter(), collections.Counter()
for row in rows:
    if row[4] != "0":
        continue
    mutant = mutants[int(row[0])]
    path = mutant["file"]
    text = texts.setdefault(path, open(os.path.join(root, path)).read())
    if path not in tables:
        tables[path] = run.functions(text)
    lines = text.split("\n")
    offset = sum(len(l) + 1 for l in lines[: mutant["start"][0] - 1]) + mutant["start"][1] - 1
    inside = any(item <= offset < end for _, item, _, end in tables[path])
    where = "inside-fn" if inside else "outside-fn"
    by_package[(mutant["package"], where)] += 1
    by_outcome[(where, row[6])] += 1
for (package, where), count in sorted(by_package.items()):
    print(f"ZERO-COVERAGE package={package} {where}={count}")
for (where, outcome), count in sorted(by_outcome.items()):
    print(f"ZERO-COVERAGE {where} library-outcome={outcome} count={count}")
