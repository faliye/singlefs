#!/usr/bin/env python3
"""把 S6SPLIT 行按 (运行, 域) 汇成两种写法的判定表。
当前判据 J6 前提真：第二次调用不是「返回根且全字段相等」即违例。三种写法：
  P   前提加「第一次落到的根 txg 小于 u64 最大值」：顶格的态不判；
  Qn  顶格的第二次报 chosen_root_txg_has_no_successor 且一条没施加 ⇒ 算相等；
  Qw  第二次报任何错（不含 panic）⇒ 算相等。
"""
import re, sys, glob, collections
rows = collections.defaultdict(lambda: collections.Counter())
verdict = {}
for path in sorted(glob.glob(sys.argv[1] + "/*.out")):
    run = path.split("/")[-1][:-4]
    for line in open(path):
        m = re.match(r"S6SPLIT name=verdict domain=(\S+) premise_holds=(\w+) first_landed_at_u64_max=(\w+) second=(\S+) states=(\d+)", line)
        if m:
            domain, prem, top, kind, n = m.groups()
            rows[(run, domain)][(prem, top, kind)] += int(n)
        m = re.match(r"E7RESULT name=verdict domain=(\S+) .*?j6_premise_holds_violations=(\d+)", line)
        if m and "set=" not in line:
            verdict[(run, m.group(1))] = int(m.group(2))
print("run\tdomain\tjudged_premise_true\tviol_now\tviol_P\tviol_Qn\tviol_Qw\tverdict_row_j6_premise_holds_violations\tviolating_kinds(top=T/F)")
for (run, domain), c in sorted(rows.items()):
    judged = sum(n for (p, t, k), n in c.items() if p == "true")
    now = sum(n for (p, t, k), n in c.items() if p == "true" and k != "equal")
    P = sum(n for (p, t, k), n in c.items() if p == "true" and k != "equal" and t == "false")
    Qn = sum(n for (p, t, k), n in c.items() if p == "true" and k not in ("equal", "failed_no_successor"))
    Qw = sum(n for (p, t, k), n in c.items() if p == "true" and k not in ("equal",) and not k.startswith("failed"))
    kinds = ";".join(f"{k}/{'T' if t == 'true' else 'F'}={n}" for (p, t, k), n in sorted(c.items()) if p == "true" and k != "equal")
    print(f"{run}\t{domain}\t{judged}\t{now}\t{P}\t{Qn}\t{Qw}\t{verdict.get((run, domain), '-')}\t{kinds}")
