#!/usr/bin/env python3
"""核「七·6 第三轮」起头的 19 行变异：原文在被改文件里恰好命中几次、替换文与原文是否不同、点名的测试在测试文件里有没有。
用法：check_mutation_rows.py <冻结副本根> <before 副本根>
before 副本 = 冻结副本去掉后续批补丁之后的树（用来看原文在改前是否也命中）。只读。"""
import sys, re, pathlib

frozen = pathlib.Path(sys.argv[1])
before = pathlib.Path(sys.argv[2]) if len(sys.argv) > 2 else None
rows = [l.rstrip("\n").split("\t") for l in (frozen / "crates/mutations.tsv").read_text().splitlines()
        if l.startswith("七·6 第三轮")]
test_text = (frozen / "crates/singlefs-harness/tests/publish_lookup_by_identity.rs").read_text()
print(f"rows={len(rows)}")
seen = set()
for i, r in enumerate(rows, 1):
    name, file, orig, repl, args, test = r
    o = orig.replace("\\n", "\n")
    n = repl.replace("\\n", "\n")
    text = (frozen / file).read_text()
    hits = text.count(o)
    hits_before = (before / file).read_text().count(o) if before else None
    test_defined = len(re.findall(r"fn " + re.escape(test) + r"\(", test_text))
    key = (file, orig, repl)
    dup = key in seen
    seen.add(key)
    print(f"{i:2d} hits={hits} hits_before={hits_before} differs={o != n} test_defined={test_defined} "
          f"args_names_test={test in args} dup={dup} {name[:40]}")
