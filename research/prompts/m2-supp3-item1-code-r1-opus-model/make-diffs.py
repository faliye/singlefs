#!/usr/bin/env python3
"""把 mutants.tsv / probes.tsv / rows-41-121.tsv 每一行在开工快照的仓副本上施加一次，输出 diff -u（副本不改：在临时目录里做）。
用法：python3 make-diffs.py <开工快照的仓副本> <表…>  > 输出.patch
"""
import os, subprocess, sys, tempfile
root = sys.argv[1]
for table in sys.argv[2:]:
    for line in open(table, encoding="utf-8"):
        line = line.rstrip("\n")
        if not line or line.startswith("#"):
            continue
        ident, name, path, old, new = line.split("\t")
        old, new = old.replace("\\n", "\n"), new.replace("\\n", "\n")
        text = open(os.path.join(root, path), encoding="utf-8").read()
        assert text.count(old) == 1, (ident, text.count(old))
        with tempfile.TemporaryDirectory() as work:
            a = os.path.join(work, "a"); b = os.path.join(work, "b")
            open(a, "w", encoding="utf-8").write(text)
            open(b, "w", encoding="utf-8").write(text.replace(old, new))
            diff = subprocess.run(["diff", "-u", "--label", f"a/{path}", "--label", f"b/{path}", a, b], capture_output=True, text=True).stdout
        print(f"### {ident}：{name}")
        print(diff, end="")
