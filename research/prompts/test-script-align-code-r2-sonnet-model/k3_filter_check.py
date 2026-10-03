#!/usr/bin/env python3
"""J3 补核：变异表里 `-- <筛选词>` 的行，筛选词在目标里匹配到几条用例（近似：数 fn 名里含筛选词的个数）。
门禁 checker-tier-crates-mutation-replay 要求点名的测试在没改坏的副本上「恰好对上一个用例」。
--test X：数 crates/<包>/tests/X.rs（和它引的子目录模块 tests/X/ 下）里 fn 名含筛选词的条数。--lib：数该包 src 下所有 .rs。--bin B：数 src/bin/B.rs。
只读，近似判（宏生成的用例、cfg 掉的用例数不出）；筛选词是子串匹配（libtest 默认），不带 --exact 的才按子串数。"""
import os, re, sys, glob
TREE = os.environ.get("K3_TREE", "/tmp/claude-1000/test-script-align-code-r2/tree")
OLD = os.environ.get("K3_OLD_TABLE", "/tmp/claude-1000/test-script-align-code-r1/tree/crates/mutations.tsv")
lines = open(f"{TREE}/crates/mutations.tsv", encoding="utf-8").read().split("\n")
old = set(open(OLD, encoding="utf-8").read().split("\n"))
bad = checked = 0
for i, l in enumerate(lines, 1):
    if not l.strip() or l.startswith("#") or l in old: continue
    name, path, orig, repl, args, test = l.split("\t")
    m_pkg = re.search(r"-p\s+(\S+)", args); pkg = m_pkg.group(1) if m_pkg else None
    if "--" not in args.split(" -- ")[0:1] and " -- " not in args:
        filt = None
    else:
        filt = args.split(" -- ", 1)[1].split()[0] if " -- " in args else None
    if not pkg or not filt: 
        print(f"SKIP row {i}: pkg={pkg} filter={filt} :: {name[:50]}"); continue
    exact = "--exact" in args
    files = []
    m_t = re.search(r"--test\s+(\S+)", args); m_b = re.search(r"--bin\s+(\S+)", args)
    if m_t:
        files = [f"{TREE}/crates/{pkg}/tests/{m_t.group(1)}.rs"] + glob.glob(f"{TREE}/crates/{pkg}/tests/{m_t.group(1)}/**/*.rs", recursive=True)
    elif m_b:
        files = [f"{TREE}/crates/{pkg}/src/bin/{m_b.group(1)}.rs"]
    elif "--lib" in args:
        files = glob.glob(f"{TREE}/crates/{pkg}/src/**/*.rs", recursive=True)
    else:
        print(f"SKIP row {i}: no --test/--lib/--bin"); continue
    names = []
    for f in files:
        if not os.path.exists(f): continue
        src = open(f, encoding="utf-8").read()
        for m in re.finditer(r"#\[test\][^\n]*\n(?:\s*#\[[^\n]*\n)*\s*(?:pub\s+)?fn\s+([A-Za-z0-9_]+)", src):
            names.append(m.group(1))
    tail = filt.split("::")[-1]  # 筛选词带模块路径时（`mod::tests::名字`）只按最后一截数；模块路径本身不核
    hit = [n for n in names if (n == tail if exact else tail in n)]
    ok = len(hit) == 1 and hit[0] == test
    checked += 1
    if not ok:
        bad += 1
        print(f"BAD row {i} filter={filt!r} matches={len(hit)} {hit[:4]} named={test!r} :: {name[:60]}")
print(f"filter_rows_checked={checked} bad={bad}")
