#!/usr/bin/env python3
"""C479-丙 的对照：父版本一律取 txg-1（不按谱系配父）时，合法历史上误红多少个评估时刻。"""
import random
import legal_history_model as m

bad = tot = 0
first = None
for seed in range(3000):
    rng = random.Random(seed)
    w = m.World(rng)
    ops = []
    for _ in range(10):
        op = rng.choice(["create", "create", "unlink", "unlink", "rollback", "crash"])
        ok = {"create": w.create, "unlink": w.unlink, "rollback": w.rollback, "crash": w.crash_then_publish}[op]()
        if ok is False:
            continue
        ops.append(op)
        tot += 1
        cand = [r for i, r in enumerate(w.ring) if i not in w.abandoned]
        by = {v.txg: v for v in cand}
        red = False
        for v in cand:
            p = by.get(v.txg - 1)
            if p is None:
                continue
            pn = {n: b for n, b, _ in p.records}
            if any((n not in pn and n < p.wm) or (n in pn and pn[n] != b) for n, b, _ in v.records):
                red = True
        if red:
            bad += 1
            if first is None or len(ops) < len(first[1]):
                first = (seed, list(ops))
print(f"评估时刻 {tot}；丙 父=txg-1（不按谱系配）误红 {bad}；最短反例 {first}")
