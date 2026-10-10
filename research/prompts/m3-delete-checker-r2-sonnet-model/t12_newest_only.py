#!/usr/bin/env python3
"""T12：I-9.6 的「树内最大 key」取候选集任一版（今天的代码）与只取最新根，在合法历史与水位压回的坏历史上各红多少个评估时刻。
窗口取全部未被抛弃的根（候选集的上限）；水位那一半（> 所选根墓碑最大对象 ID）两边相同，不比。"""
import random
import sys

sys.argv = ["x", "3000", "10", "refuse"]
import bad_history_model as b
import legal_history_model as m


def reds(world):
    sel = world.ring[world.newest()]
    cand = [r for i, r in enumerate(world.ring) if i not in world.abandoned]
    cross = not (sel.wm > max((n for v in cand for n, _, _ in v.records), default=0))
    newest_only = not (sel.wm > max((n for n, _, _ in sel.records), default=0))
    return cross, newest_only


def run(world_class, ops_names, press_allowed):
    total = cross_red = newest_red = cross_only = 0
    for seed in range(3000):
        rng = random.Random(seed)
        w = world_class(rng)
        for _ in range(10):
            op = rng.choice(ops_names)
            if op == "press":
                ok = b.press(w)
            else:
                ok = {"create": w.create, "unlink": w.unlink, "rollback": w.rollback, "crash": w.crash_then_publish}[op]()
            if ok is False:
                continue
            total += 1
            c, n = reds(w)
            cross_red += int(c)
            newest_red += int(n)
            cross_only += int(c and not n)
    return total, cross_red, newest_red, cross_only


legal = run(m.World, ["create", "create", "unlink", "unlink", "rollback", "crash"], False)
bad = run(b.BadWorld, ["create", "create", "unlink", "unlink", "rollback", "crash", "press"], True)
print(f"合法历史：评估时刻 {legal[0]}，跨根取红 {legal[1]}，只取最新根红 {legal[2]}，只有跨根红 {legal[3]}")
print(f"含水位压回：评估时刻 {bad[0]}，跨根取红 {bad[1]}，只取最新根红 {bad[2]}，只有跨根红 {bad[3]}")
