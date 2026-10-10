#!/usr/bin/env python3
"""对副本临时施加 / 还原 mutations.tsv 里点名的变异行（行首若干字取名字前缀，各命中恰好一行），只在自己草稿目录的副本里写。
用法 patch_b35.py <副本根> <冻结副本根> apply <名字前缀> [<名字前缀> …] | restore
restore：把各被改过的源文件从冻结副本拷回并核 sha256。"""
import hashlib, shutil, sys
root, frozen, action = sys.argv[1], sys.argv[2], sys.argv[3]
tsv = [l.split("\t") for l in open(f"{frozen}/crates/mutations.tsv", encoding="utf-8").read().split("\n") if l.count("\t") == 5]
touched = f"{root}/.patch_b35_touched"
if action == "restore":
    bad = 0
    for rel in sorted(set(open(touched).read().split())) if __import__("os").path.exists(touched) else []:
        shutil.copyfile(f"{frozen}/{rel}", f"{root}/{rel}")
        same = hashlib.sha256(open(f"{root}/{rel}", "rb").read()).hexdigest() == hashlib.sha256(open(f"{frozen}/{rel}", "rb").read()).hexdigest()
        print("restore", rel, "ok" if same else "MISMATCH"); bad += 0 if same else 1
    __import__("os").path.exists(touched) and __import__("os").remove(touched)
    sys.exit(bad)
for prefix in sys.argv[4:]:
    rows = [r for r in tsv if r[0].startswith(prefix)]
    assert len(rows) == 1, (prefix, len(rows))
    name, source, find, repl, args, test = rows[0]
    path = f"{root}/{source}"
    s = open(path, encoding="utf-8").read()
    f, g = find.replace("\\n", "\n"), repl.replace("\\n", "\n")
    assert s.count(f) == 1, (name, s.count(f))
    open(path, "w", encoding="utf-8").write(s.replace(f, g))
    open(touched, "a").write(source + "\n")
    print("applied", name[:60])
