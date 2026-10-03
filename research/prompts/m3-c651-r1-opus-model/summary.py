#!/usr/bin/env python3
"""每个候选：全池槽 0 都不可用的崩溃态（C 组：新盘纯崩溃；S 组：旧盘重做 mkfs 再崩溃）里，结局按类计态数。
类：ok = 实现择到本池槽 1、oracle 过、checker 读到本池槽 1 且不判红；stale_mounted = 实现择到旧池残留槽而恢复没报错（oracle 过或不过）；
refused = 实现整池拒或恢复失败；checker_only = 实现 ok 而 checker 读到的不是本池槽 1 或判红。用法：python3 tabulate.py … | python3 summary.py"""
import collections, re, sys
count = collections.defaultdict(collections.Counter)
for line in sys.stdin:
    if not line.startswith("cell "):
        continue
    label = re.search(r"label=(\S+)", line).group(1)
    h = re.search(r" h=(\S+)", line).group(1)
    group = h[0]
    body = line.split(f" h={h} ", 1)[1].strip()
    for part in body.split(" ; "):
        core, oracle, chk, red, n = re.match(r"core=(\S+) oracle=(\S+) checker=(\S+) red=(\[.*?\]) x(\d+)", part).groups()
        n = int(n)
        chk_ok = all(p.startswith("new") for p in chk.split("|")) and red == "[]"
        if core.startswith("new") and oracle == "-":
            kind = "ok" if chk_ok else "checker_only"
        elif core.startswith("stale") and not oracle.startswith(("NoRootChosen", "ReadFailed")):
            kind = "stale_mounted"
        else:
            kind = "refused"
        count[(label, group)][kind] += n
for (label, group), c in sorted(count.items()):
    total = sum(c.values())
    print(f"summary label={label} group={group} states={total} " + " ".join(f"{k}={c[k]}" for k in ["ok", "checker_only", "stale_mounted", "refused"]))
