#!/usr/bin/env python3
# D5（快照 / 空间记账机制） 未定项 12 第三轮攻方腿模型：搜索、回放、随机历史与缩小。
#   BFS 按层、状态去重；队列只存「父节点号 + 操作」，展开时从根重放那段历史，去重用 16 字节摘要（第二轮 bfs-big 峰值 6.4 GB）。
#   真值 dl2.oracle：多放与重复放任何时刻判，漏放只在臂没有未完成意图时判。
import random
import dl2 as base
from dl3 import clone_state
from dl3_world import new_state, step, legal_ops, digest

label = base.label
WEIGHTS3 = dict(base.WEIGHTS, BA=0.8, BS=0.8, BG=0.4, RBC=0.15, PF=0.25)
BATCH_KINDS = ("B", "BA", "BS", "BG")


def replay_state(arm_specs, path, keep_ring):
    state = new_state(arm_specs, keep_ring)
    for op in path:
        step(state, tuple(op))
    return state


def bfs_shortest(arm_name, cfg, limits, max_depth, budget=2000000):
    keep_ring = bool(limits.get("crash") or limits.get("rbc"))
    spec = [(arm_name, cfg)]
    parents, ops_of, frontier = [-1], [None], [0]
    seen, explored = {digest(new_state(spec, keep_ring))}, 0

    def path_of(node):
        path = []
        while node > 0:
            path.append(ops_of[node])
            node = parents[node]
        return tuple(reversed(path))
    for depth in range(max_depth):
        upcoming = []
        for node in frontier:
            path = path_of(node)
            state = replay_state(spec, path, keep_ring)
            for op in legal_ops(state, limits):
                child = clone_state(state)
                step(child, op)
                explored += 1
                over, double, leak = base.oracle(child["world"], child["arms"][0])
                if over or double or leak:
                    return {"arm": arm_name, "cfg": cfg, "found": True, "path": path + (op,), "over": over,
                            "double": double, "leak": leak, "explored": explored, "states": len(seen)}
                key = digest(child)
                if key not in seen:
                    seen.add(key)
                    parents.append(node)
                    ops_of.append(op)
                    upcoming.append(len(parents) - 1)
                if explored >= budget:
                    return {"arm": arm_name, "cfg": cfg, "found": False, "explored": explored, "states": len(seen),
                            "depth_reached": depth, "truncated": True}
        frontier = upcoming
        if not frontier:
            break
    return {"arm": arm_name, "cfg": cfg, "found": False, "explored": explored, "states": len(seen),
            "depth_reached": max_depth, "truncated": False}


def drain(state, rng, limit=100000):
    """把剩下的意图做完（rng 为 None 时每次取第一个可动意图）；分到下一次发布的 prev 没传完就再发布一次。"""
    for _ in range(limit):
        arms = state["arms"]
        if any(arm.intents for arm in arms):
            step(state, ("B", rng.randrange(1 << 16) if rng else 0))
        elif any(arm.pending for arm in arms):
            step(state, ("P",))
        else:
            return True
    return False


def replay_summary(arm_specs, path):
    """同一段历史喂给几条臂：每条臂真值第一次判红在第几步；历史走完后把意图做完再判一次。"""
    state, result = new_state(arm_specs), {}
    for number, op in enumerate(path, 1):
        step(state, tuple(op))
        for (name, cfg), arm in zip(arm_specs, state["arms"]):
            entry = result.setdefault(label(name, cfg), {"bad": None})
            over, double, leak = base.oracle(state["world"], arm)
            if entry["bad"] is None and (over or double or leak):
                entry["bad"] = {"step": number, "over": over, "double": double, "leak": leak}
    drain(state, None)
    for (name, cfg), arm in zip(arm_specs, state["arms"]):
        entry = result[label(name, cfg)]
        over, double, leak = base.oracle(state["world"], arm)
        if entry["bad"] is None and (over or double or leak):
            entry["bad"] = {"step": "drain", "over": over, "double": double, "leak": leak}
    return result


def trace(arm_specs, path):
    state, lines = new_state(arm_specs), []
    shown = ("kind", "seq", "head", "lo", "hi", "P", "src", "dst", "filt", "phase")
    for number, op in enumerate(path, 1):
        step(state, tuple(op))
        for (name, cfg), arm in zip(arm_specs, state["arms"]):
            over, double, leak = base.oracle(state["world"], arm)
            keys = sorted(arm.ks.entries) if name != "REF" else sorted((o, sorted(v)) for o, v in arm.dl.items())
            intents = [{k: v for k, v in intent.items() if k in shown} for intent in arm.intents]
            lines.append(f"{number} {tuple(op)} | {label(name, cfg)} | dl={keys} | ph={sorted(arm.ph.items())} "
                         f"ps={sorted(arm.ps.items())} | intents={intents} | freed={sorted(arm.freed)} "
                         f"double={arm.double} | over={over} leak={leak}")
    return lines


from dl3_fuzz import pick, fuzz, record, verdict, minimize   # noqa: E402  随机历史与缩小拆在 dl3_fuzz（单个文件一次写入不超过 150 行）
