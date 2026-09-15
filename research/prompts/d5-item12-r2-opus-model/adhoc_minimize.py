#!/usr/bin/env python3
# 把 fuzz 里某条臂的某一例判红历史原样录下来，再按 ddmin 删操作缩到局部极小（删掉之后每步仍合法、末尾仍判红才留）。
# 缩出来的历史再喂给全部变体，看同一段历史打中哪几条臂（分不分辨臂）。只在本模型上量过。
import sys, json, random
import dl2 as m


def record(seed, length=150, limits=m.FUZZ, specs=m.VARIANTS):
    """与 dl2.fuzz 同一套随机数取法，录下那一段历史（含末尾把意图做完的那几批）。"""
    rng = random.Random(seed)
    state, ops = m.new_state(specs), []
    for _ in range(length):
        legal = m.legal_ops(state, limits, (1, 2, 3))
        kinds = sorted({op[0] for op in legal})
        kind = rng.choices(kinds, [m.WEIGHTS[k] for k in kinds])[0]
        op = rng.choice([candidate for candidate in legal if candidate[0] == kind])
        op = ("B", rng.randrange(1 << 16)) if kind == "B" else op
        m.step(state, op)
        ops.append(op)
    for _ in range(100000):
        if not any(arm.intents for arm in state["arms"]):
            break
        op = ("B", rng.randrange(1 << 16))
        m.step(state, op)
        ops.append(op)
    return ops


def verdict(spec, ops, limits=m.FUZZ):
    """单臂回放：非法操作 ⇒ None；没有意图时的 B 当空操作；末尾按提交序把剩下的意图做完再判真值。"""
    state = m.new_state([spec])
    for op in ops:
        if op[0] == "B":
            if state["arms"][0].intents:
                m.step(state, op)
            continue
        if op not in m.legal_ops(state, limits, (1, 2, 3)):
            return None
        m.step(state, op)
    while state["arms"][0].intents:
        m.step(state, ("B", 0))
    return m.oracle(state["world"], state["arms"][0])


def minimize(spec, ops, want):
    def holds(candidate):
        result = verdict(spec, candidate)
        return result is not None and want(result)
    assert holds(ops), "录下的历史在单臂回放里不判红"
    chunk = max(1, len(ops) // 2)
    while chunk >= 1:
        index, shrunk = 0, False
        while index < len(ops):
            candidate = ops[:index] + ops[index + chunk:]
            if holds(candidate):
                ops, shrunk = candidate, True
            else:
                index += chunk
        if not shrunk:
            chunk //= 2
    return ops


def main():
    cases = [(("BING_R", {"order": "any"}), 11), (("YI", {"order": "any"}), 10), (("BING_T", {"order": "any"}), 10)]
    for spec, seed in cases:
        ops = record(seed)
        want = (lambda result: bool(result[2])) if verdict(spec, ops)[2] else (lambda result: bool(result[0]))
        small = minimize(spec, ops, want)
        print(json.dumps({"arm": m.label(*spec), "seed": seed, "recorded": len(ops), "minimized": small,
                          "result": verdict(spec, small),
                          "all_variants": m.replay_summary(m.VARIANTS, small)}, ensure_ascii=False, default=str), flush=True)


if __name__ == "__main__":
    main()
