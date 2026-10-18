"""汇总 run 的输出：走到那一判的行按（历史，读故障，次序，R 此前，判，结局类别）计数；R 此前分 walkable（状态不红 I-7.2）/ gone（红 I-7.2）。
打中 = 判放过、挂载做成、写行根之前新写掉 R 的单元（new>0）、而状态不红 I-7.2（R 此前走得完）。
用法：python3 aggregate.py <输出文件…>"""
import re, sys, collections
cells = collections.Counter()
examples = {}
not_reached = collections.Counter()
for path in sys.argv[1:]:
    for line in open(path):
        if line.startswith("name=k1_state"):
            f = dict(piece.split("=", 1) for piece in line.split() if "=" in piece)
            before = "gone" if "I-7.2" in f["state_red"] else "walkable"
            judge = f["judge"].split("[")[0]
            out = f["outcome"]
            if out.startswith("mounted"):
                new = int(re.search(r":new=(\d+)", out).group(1))
                kind = "mounted_overwrote" if new > 0 else "mounted_r_kept"
            else:
                kind = ":".join(out.split(":")[:3])
            key = (f["history"], f["faults"], f["order"], before, judge, kind)
            cells[key] += 1
            examples.setdefault(key, f["ordinal"])
        elif line.startswith("name=k1_not_reached"):
            f = dict(piece.split("=", 1) for piece in line.split() if "=" in piece)
            not_reached[(f["history"], f["faults"], f["order"], f["outcome"])] += int(f["states"])
for key in sorted(cells):
    history, faults, order, before, judge, kind = key
    hit = " HIT" if (judge == "pass" and kind == "mounted_overwrote" and before == "walkable") else ""
    print(f"name=k1_cell history={history} faults={faults} order={order} before={before} judge={judge} outcome={kind} states={cells[key]} example_ordinal={examples[key]}{hit}")
for key in sorted(not_reached):
    history, faults, order, outcome = key
    print(f"name=k1_not_reached_total history={history} faults={faults} order={order} outcome={outcome} states={not_reached[key]}")
