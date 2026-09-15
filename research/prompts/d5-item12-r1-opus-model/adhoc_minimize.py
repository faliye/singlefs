#!/usr/bin/env python3
# 把 fuzz 里某条臂第一例判红的历史按同一个随机数序列复原，再逐条删操作（删了仍判红就删），得到局部最短的历史。
import json, random, sys
import dlmodel as m


def regenerate(seed, length, limits, classes=(1, 2, 3)):
    rng, world, ops = random.Random(seed), m.World(), []
    for _ in range(length):
        legal = m.legal_ops(world, limits, classes)
        if not legal:
            break
        kinds = sorted({op[0] for op in legal})
        kind = rng.choices(kinds, [m.WEIGHTS[k] for k in kinds])[0]
        op = rng.choice([candidate for candidate in legal if candidate[0] == kind])
        m.step(world, [], op)
        ops.append(op)
    return ops


def fails(arm_name, ops, limits, want, classes=(1, 2, 3)):
    world, arm = m.World(), None
    arm = m.ARMS[arm_name](world)
    for op in ops:
        if op not in m.legal_ops(world, limits, classes):
            return False            # 删掉一步之后变得不合法的历史不算
        m.step(world, [arm], op)
        over, double, leak = m.oracle(world, arm)
        if (want == "over" and over) or (want == "leak" and leak):
            return True
    return False


def minimize(arm_name, ops, limits, want):
    changed = True
    while changed:
        changed = False
        for position in range(len(ops) - 1, -1, -1):
            candidate = ops[:position] + ops[position + 1:]
            if fails(arm_name, candidate, limits, want):
                ops, changed = candidate, True
    return ops


for arm_name, seed, stop, want in (("JIA_B", 0, 73, "over"), ("BING_B", 1, 107, "over")):
    ops = regenerate(seed, 120, m.FUZZ)[:stop]
    assert fails(arm_name, ops, m.FUZZ, want), "复原的历史不判红：随机数序列对不上"
    short = minimize(arm_name, ops, m.FUZZ, want)
    print(json.dumps({"section": "minimized", "arm": arm_name, "seed": seed, "want": want,
                      "original_len": len(ops), "min_len": len(short), "ops": short}))
    for line in m.replay([arm_name, "JIA", "YI_min", "BING_R"], short):
        print(json.dumps({"section": "replay", "arm": arm_name, "line": line}, ensure_ascii=False))
print(json.dumps({"section": "END"}))
