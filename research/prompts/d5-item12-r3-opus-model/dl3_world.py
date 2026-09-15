#!/usr/bin/env python3
# D5（快照 / 空间记账机制） 未定项 12 第三轮攻方腿模型：世界操作与合法操作。
#   发布：一次发布整体施加（D16 已定项 4），根环留 3 个根；分到下一次发布才传的 prev 在发布之后施加（根里带着没传的那几笔）。
#   PF：根槽写失败，这个 checkpoint 推进一格再发（D23 已定项 14）——已写的块 birth 仍是原号，快照与根取推进后的号，意图里记的上界不变。
#   CR：崩溃挂回最新的根。RB：回退一个根，新 txg = 根环全部根 txg 的 max + 1、树 ID 水位取全部根的 max（D23 已定项 14、D8 已定项 8 ② 的字面）。
#   RBC：同 RB，但按误读只取候选根（被抛弃的不算）⇒ 重发被抛弃时间线用过的 txg 与树 ID。
#   B / BA / BS / BG：一条 / 顺序施加的一批 / 计划后不复核的一批 / 计划后复核的一批。
import hashlib
import dl2 as base
from dl3 import ARMS3, STRONG3, clone_state


def new_state(arm_specs, keep_ring=True):
    world = base.World()
    arms = [ARMS3[name](world, dict(STRONG3, **cfg)) for name, cfg in arm_specs]
    return {"world": world, "arms": arms, "ring": [], "keep_ring": keep_ring}


def publish(state):
    world = state["world"]
    published_txg = world.txg
    world.txg += 1
    if state["keep_ring"]:
        state["ring"].append({"copy": clone_state((world, state["arms"])), "txg": published_txg,
                              "tid": world.next_tid, "abandoned": False})
        state["ring"] = state["ring"][-3:]
    for arm in state["arms"]:
        arm.apply_pending_passes()


def restore(state, entry, watermark_from):
    world, arms = clone_state(entry["copy"])
    world.txg = max(item["txg"] for item in watermark_from) + 1
    world.next_tid = max(item["tid"] for item in watermark_from)
    state["world"], state["arms"] = world, arms
    for arm in arms:
        if arm.cfg["inherit"] == "nextpub":
            arm.apply_pending_passes()          # 恢复时照意图补传
        elif arm.cfg["inherit"] == "nextpub_lost":
            arm.drop_pending_passes()           # 恢复时不补传


def step(state, op):
    world, arms, kind = state["world"], state["arms"], op[0]
    if kind in ("W", "K", "C", "D", "X"):
        base.step(state, op)
    elif kind in ("P", "PS", "PSS", "PSC", "PF"):
        sub, head = (op[1], op[2]) if kind == "PF" else (kind, op[1] if len(op) > 1 else None)
        if kind == "PF":
            world.txg += 1
        if sub != "P":
            origin = base.take_snapshot(world, arms, head)
        if sub == "PSS":
            base.take_snapshot(world, arms, head)
        if sub == "PSC":
            base.take_snapshot(world, arms, base.make_clone(world, arms, origin))
        publish(state)
    elif kind == "B":
        for arm in arms:
            eligible = arm.eligible()
            if eligible:
                arm.run_unit(world, eligible[op[1] % len(eligible)])
    elif kind == "BA":
        for arm in arms:
            arm.run_batch_seq(world, op[1])
    elif kind in ("BS", "BG"):
        for arm in arms:
            arm.run_batch_snap(world, op[1], kind == "BG")
    elif kind == "CR":
        alive = [entry for entry in state["ring"] if not entry["abandoned"]]
        restore(state, alive[-1], state["ring"])
    elif kind in ("RB", "RBC"):
        alive = [entry for entry in state["ring"] if not entry["abandoned"]]
        alive[-1]["abandoned"] = True
        restore(state, alive[-2], state["ring"] if kind == "RB" else alive[:-1])
    else:
        raise ValueError(op)


def legal_ops(state, limits, classes=(1,)):
    world, arms, ops = state["world"], state["arms"], []
    for head in sorted(world.heads):
        if len(world.blocks) < limits["blocks"]:
            ops += [("W", head, cls) for cls in classes]
        ops += [("K", head, bid) for bid in sorted(world.heads[head]["live"])]
    if world.txg < limits["txg"]:
        ops.append(("P",))
        resend = limits.get("pf") and world.txg + 1 < limits["txg"]
        if resend:
            ops.append(("PF", "P", None))
        for head in sorted(world.heads):
            if len(world.snaps) + 1 <= limits["snaps"]:
                ops.append(("PS", head))
                if resend:
                    ops.append(("PF", "PS", head))
            if limits.get("pss") and len(world.snaps) + 2 <= limits["snaps"]:
                ops.append(("PSS", head))
            if limits.get("psc") and len(world.snaps) + 2 <= limits["snaps"] and len(world.heads) < limits["heads"]:
                ops.append(("PSC", head))
    for sid in world.alive_snaps():
        if not world.clones_of(sid):                       # 分叉点闸：children 数克隆
            ops.append(("D", sid))
        if limits.get("clone") and len(world.heads) < limits["heads"]:
            ops.append(("C", sid))
    for head in sorted(world.heads):                      # 第二道闸：有自己活快照的头不许销毁
        if limits.get("x", True) and head != world.root and not world.head_snaps(head):
            ops.append(("X", head))
    width = max(len(arm.eligible()) for arm in arms)
    ops += [("B", index) for index in range(width)]
    if width:
        ops += [tuple(item) for item in limits.get("batch_ops", ())]
    alive = sum(1 for entry in state["ring"] if not entry["abandoned"])
    if state["ring"] and limits.get("crash"):
        ops.append(("CR",))
        if alive >= 2:
            ops.append(("RB",))
    if alive >= 2 and limits.get("rbc"):
        ops.append(("RBC",))
    return ops


def arm_sig(arm):
    return base.arm_sig(arm) + (repr(arm.pending),)


def digest(state):
    """状态去重用的 16 字节摘要：世界、每条臂（含没传的 prev）、根环（含每个根的世界与臂）。"""
    parts = (base.world_sig(state["world"]), tuple(arm_sig(arm) for arm in state["arms"]))
    if state["keep_ring"]:
        parts += (tuple((entry["txg"], entry["tid"], entry["abandoned"], base.world_sig(entry["copy"][0]),
                         tuple(arm_sig(arm) for arm in entry["copy"][1])) for entry in state["ring"]),)
    return hashlib.blake2b(repr(parts).encode(), digest_size=16).digest()
