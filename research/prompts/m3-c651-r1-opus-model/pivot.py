#!/usr/bin/env python3
"""把 tabulate.py 的 cell 行压成短码、按历史把各候选并排，再把并排结果相同的历史归成一组。
短码（每个全池槽 0 都不可用的崩溃态一个，多个态用 + 连）：实现 N=择到本池槽 1、S=择到旧池残留槽、R=整池拒；
oracle 过了不标、不过标 !<种类>；checker 每盘一个字母 N/S/0（0=没读到可用的槽）；checker 判红的不变量写在 [] 里。
用法：python3 tabulate.py … | python3 pivot.py"""
import collections, re, sys

short_oracle = {"NoRootChosen": "!noroot", "ReadFailed": "!fail", "WrongContent": "!wrong",
                "FileReadWhereThisRootHasNone": "!fileNone", "NoFileWhereThisRootHasOne": "!nofile",
                "RecoveredToAnOlderRoot": "!older", "NoFileWhereAnOlderRootHasOne": "!nofileOlder"}
rows = collections.defaultdict(dict)
labels = []
for line in sys.stdin:
    if not line.startswith("cell "):
        continue
    label = re.search(r"label=(\S+)", line).group(1)
    if label not in labels:
        labels.append(label)
    h = re.search(r" h=(\S+)", line).group(1)
    body = line.split(f" h={h} ", 1)[1].strip()
    codes = []
    for part in body.split(" ; "):
        m = re.match(r"core=(\S+) oracle=(\S+) checker=(\S+) red=(\[.*?\]) x(\d+)", part)
        core, oracle, chk, red, n = m.groups()
        c = {"refuse": "R"}.get(core, "N" if core.startswith("new") else "S" if core.startswith("stale") else "?")
        o = "" if oracle == "-" else short_oracle.get(oracle, "!" + oracle)
        k = "".join("N" if p.startswith("new") else "S" if p.startswith("stale") else "0" for p in chk.split("|"))
        r = "" if red == "[]" else red.replace('"', "")
        codes.append(f"{c}{o}/{k}{r}" + (f"x{n}" if n != "1" else ""))
    rows[h][label] = "+".join(sorted(codes))
groups = collections.defaultdict(list)
for h, by_label in rows.items():
    groups[tuple(by_label.get(l, "-") for l in labels)].append(h)
print("| 历史（组内条数） | " + " | ".join(labels) + " |")
print("|---|" + "---|" * len(labels))
for sig, hs in sorted(groups.items(), key=lambda kv: kv[1][0]):
    name = hs[0] if len(hs) == 1 else f"{hs[0]} 等 {len(hs)} 段"
    print(f"| {name} | " + " | ".join(sig) + " |")
print(f"histories={len(rows)} groups={len(groups)}")
