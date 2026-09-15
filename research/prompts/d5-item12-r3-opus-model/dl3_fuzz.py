#!/usr/bin/env python3
# D5（快照 / 空间记账机制） 未定项 12 第三轮攻方腿模型：随机历史、录历史、单臂判定与 ddmin 缩小。
# 从 dl3_search 拆出来（单个文件一次写入不超过 150 行）；dl3_search 末尾把这几个名字原样导出，入口 dl3_main 不用改。
# 用到 dl3_search 的 drain / label / WEIGHTS3 / BATCH_KINDS 时在函数里现取，免得两个模块互相导入。
import random
import dl2 as base
from dl3_world import new_state, step, legal_ops


def pick(rng, state, limits, classes):
    import dl3_search as search
    ops = legal_ops(state, limits, classes)
    kinds = sorted({op[0] for op in ops})
    kind = rng.choices(kinds, [search.WEIGHTS3[k] for k in kinds])[0]
    return rng.choice([op for op in ops if op[0] == kind])


def fuzz(arm_specs, seeds, length, limits, classes=(1, 2, 3)):
    """随机历史（含克隆、销毁头、崩溃、回退、重发、各种批）：每步判真值；末尾把意图做完再判。"""
    import dl3_search as search
    labels = [search.label(name, cfg) for name, cfg in arm_specs]
    summary = {lab: dict(bad=0, over=0, double=0, leak=0, first=None) for lab in labels}
    for seed in range(seeds):
        rng, state = random.Random(seed), new_state(arm_specs)
        bad = [None] * len(labels)
        for index in range(length + 1):
            if index < length:
                step(state, pick(rng, state, limits, classes))
            else:
                search.drain(state, rng)
            for position, arm in enumerate(state["arms"]):
                over, double, leak = base.oracle(state["world"], arm)
                if bad[position] is None and (over or double or leak):
                    bad[position] = (index, bool(over), bool(double), bool(leak))
        for position, lab in enumerate(labels):
            if bad[position] is not None:
                entry = summary[lab]
                entry["bad"] += 1
                entry["over"] += bad[position][1]
                entry["double"] += bad[position][2]
                entry["leak"] += bad[position][3]
                entry["first"] = entry["first"] or {"seed": seed, "step": bad[position][0]}
    return summary


def record(arm_specs, seed, length, limits, classes=(1, 2, 3)):
    """与 fuzz 同一套随机数取法，录下那一段历史（含末尾把意图做完、补发布的那几步）。"""
    rng, state, ops = random.Random(seed), new_state(arm_specs), []
    for _ in range(length):
        op = pick(rng, state, limits, classes)
        step(state, op)
        ops.append(op)
    for _ in range(100000):
        if any(arm.intents for arm in state["arms"]):
            op = ("B", rng.randrange(1 << 16))
        elif any(arm.pending for arm in state["arms"]):
            op = ("P",)
        else:
            break
        step(state, op)
        ops.append(op)
    return ops


def verdict(spec, ops, limits, classes=(1, 2, 3)):
    """单臂回放：非法操作 ⇒ None；批类操作在没有可动意图时当空操作；返回（多放过、重复放过、漏放过）三个布尔，每步都判。"""
    import dl3_search as search
    state, flags = new_state([spec]), [False, False, False]

    def judge():
        for position, value in enumerate(base.oracle(state["world"], state["arms"][0])):
            flags[position] = flags[position] or bool(value)
    for op in ops:
        if op[0] in search.BATCH_KINDS:
            if state["arms"][0].eligible():
                step(state, op)
                judge()
            continue
        if op != ("P",) and op not in legal_ops(state, limits, classes):
            return None
        step(state, op)
        judge()
    search.drain(state, None)
    judge()
    return tuple(flags)


def minimize(spec, ops, want, limits):
    """ddmin：删掉之后仍合法、仍判红才留。"""
    def holds(candidate):
        result = verdict(spec, candidate, limits)
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
