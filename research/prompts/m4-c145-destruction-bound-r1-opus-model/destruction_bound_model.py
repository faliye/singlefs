#!/usr/bin/env python3
"""攻方腿抽象模型：未完成销毁意图的区间上界（m4-c145-destruction-bound-r1，B1–B2）。

按冻结副本里 core 的写法建状态机（crates/singlefs-core/src/transaction.rs）：
- 覆盖写换下一个单元：birth ≤ prev(头) 进 deadlist（纪元 = prev(头)），否则直接释放；
- 建快照：前驱 = prev(头)，prev(头) := 这次的 txg；有销毁中的快照时拒（publish_create_snapshot）；
- 删快照意图：记号；前驱 P 传给派生链上的下一条码 14（不看记号，next_side_of_the_snapshot）或活头；有销毁中的拒；
- 一批：hi 现取（下一侧是码 14 取它的 txg，是活头取 core 变体给的值），放 [lo, hi) 里 birth > P 的条目，批大小 1，
  能放的不多于一条就是最后一批（放完、删掉快照）。
三种调度：product（会话：意图之后只走批，崩溃续做同一个计划）；transaction（直接调发布函数：意图之后可插覆盖写，
建快照与第二个意图被 core 的守卫拒）；unguarded（假想 core 没有串行守卫：意图之后什么都能插）。
每个可达状态上按池级 checker 的判法（crates/singlefs-checker/src/walk.rs 的 judge_the_deadlist_against_the_head_and_the_live_snapshots）
各取几种「下一侧是活头时的上界」判一遍，比结论；另记释放侧的错（提早释放、重复释放）与 deadlist 里纪元 ≥ 意图 txg 的条目。
只用标准库；输出 JSON 行。
"""
import itertools
import json
import sys

INF = float("inf")


class State:
    __slots__ = ("t", "head", "head_prev", "snaps", "dead", "birth", "freed", "next_unit", "bugs")

    def __init__(self, units):
        self.t = 1  # 第一个文件那次发布的 txg
        self.head = tuple(range(units))
        self.birth = {u: 1 for u in range(units)}
        self.head_prev = 0
        self.snaps = ()  # (id, txg, prev, refs, destroying, intent_txg)
        self.dead = frozenset()  # (epoch, unit)
        self.freed = frozenset()
        self.next_unit = units
        self.bugs = ()

    def copy(self):
        other = State.__new__(State)
        for name in State.__slots__:
            setattr(other, name, getattr(self, name))
        other.birth = dict(self.birth)
        return other

    def key(self):
        return (self.t, self.head, self.head_prev, self.snaps, self.dead, self.freed, self.bugs)


def live_refs(state):
    refs = set()
    for (_, _, _, srefs, destroying, _) in state.snaps:
        if not destroying:
            refs |= set(srefs)
    return refs


def free(state, unit, why):
    bugs = list(state.bugs)
    if unit in state.freed:
        bugs.append(f"double-free:{unit}:{why}")
    if unit in state.head or unit in live_refs(state):
        bugs.append(f"early-free:{unit}:{why}")
    state.freed = state.freed | {unit}
    state.bugs = tuple(bugs)


def overwrite(state, position):
    s = state.copy()
    s.t += 1
    old = s.head[position]
    new = s.next_unit
    s.next_unit += 1
    s.birth[new] = s.t
    s.head = s.head[:position] + (new,) + s.head[position + 1:]
    if s.birth[old] <= s.head_prev:
        s.dead = s.dead | {(s.head_prev, old)}
    else:
        free(s, old, "overwrite")
    return s


def create_snapshot(state, guarded, limit):
    if guarded and any(sn[4] for sn in state.snaps):
        return None
    if len(state.snaps) >= limit:
        return None
    s = state.copy()
    s.t += 1
    sid = max([sn[0] for sn in s.snaps], default=0) + 1
    s.snaps = s.snaps + ((sid, s.t, s.head_prev, tuple(sorted(s.head)), False, None),)
    s.head_prev = s.t
    return s


def chain(state):
    return sorted(state.snaps, key=lambda sn: (sn[1], sn[0]))


def destroy_intent(state, sid, guarded):
    if guarded and any(sn[4] for sn in state.snaps):
        return None
    target = [sn for sn in state.snaps if sn[0] == sid]
    if not target or target[0][4]:
        return None
    s = state.copy()
    s.t += 1
    ordered = chain(s)
    position = [sn[0] for sn in ordered].index(sid)
    me = ordered[position]
    new_snaps = []
    for sn in s.snaps:
        if sn[0] == sid:
            new_snaps.append((sn[0], sn[1], sn[2], sn[3], True, s.t))
        elif position + 1 < len(ordered) and sn[0] == ordered[position + 1][0]:
            new_snaps.append((sn[0], sn[1], me[2], sn[3], sn[4], sn[5]))
        else:
            new_snaps.append(sn)
    if position + 1 >= len(ordered):
        s.head_prev = me[2]
    s.snaps = tuple(new_snaps)
    return s


CORE_HI_WHEN_THE_NEXT_SIDE_IS_THE_HEAD = {
    # 冻结副本的实现：这一批自己的 txg（publish_snapshot_destruction_step 传 txg）。
    "batch_txg": lambda s, me: s.t + 1,
    # 「取已发布的最大号」按批现取：这一批之前那一版的 txg。
    "published_at_the_batch": lambda s, me: s.t,
    # kb 原文：写意图那次发布的 txg。
    "intent_txg": lambda s, me: me[5],
    "infinity": lambda s, me: INF,
    # 「取已发布的最大号」在写意图那一刻定死（要存进意图才做得到）：意图那次之前那一版的 txg。
    "published_at_the_intent": lambda s, me: me[5] - 1,
    # 两个不等价的对照变体（判别力用）：区间收成只剩 lo 那一格、或空区间。
    "lo_plus_one": lambda s, me: me[1] + 1,
    "lo": lambda s, me: me[1],
}


def destroy_batch(state, sid, core_hi):
    target = [sn for sn in state.snaps if sn[0] == sid]
    if not target or not target[0][4]:
        return None
    me = target[0]
    s = state.copy()
    s.t += 1
    ordered = chain(s)
    position = [sn[0] for sn in ordered].index(sid)
    if position + 1 < len(ordered):
        hi = ordered[position + 1][1]
    else:
        hi = CORE_HI_WHEN_THE_NEXT_SIDE_IS_THE_HEAD[core_hi](state, me)
    releasable = sorted(
        (e, u) for (e, u) in s.dead if me[1] <= e < hi and s.birth[u] > me[2]
    )
    final = len(releasable) <= 1
    chosen = releasable if final else releasable[:1]
    s.dead = s.dead - set(chosen)
    if final:
        s.snaps = tuple(sn for sn in s.snaps if sn[0] != sid)
    for (_, u) in chosen:
        free(s, u, f"batch-of-{sid}")
    return s


CHECKER_BOUND_WHEN_THE_NEXT_LIVE_SIDE_IS_THE_HEAD = {
    "infinity": lambda s, d: INF,  # 冻结副本的 checker
    "intent_txg": lambda s, d: d[5],  # 盘上读不出，模型里有
    "version_txg_plus_one": lambda s, d: s.t + 1,
    # 被判那一版自己的 txg：合法镜像上每条纪元 < 写它那次发布的 txg ≤ 这一版的 txg。
    "version_txg": lambda s, d: s.t,
    "lo_plus_one": lambda s, d: d[1] + 1,
}


def next_live_bound(state, txg, sid):
    for sn in chain(state):
        if not sn[4] and (sn[1], sn[0]) > (txg, sid):
            return sn[1]
    return None


def checker_verdict(state, variant):
    reds = []
    live = [sn for sn in chain(state) if not sn[4]]
    live_units = live_refs(state)
    sides = [(sn[2], sn[1], f"snap{sn[0]}") for sn in live] + [(state.head_prev, INF, "head")]

    def pending(e, u):
        for d in state.snaps:
            if not d[4]:
                continue
            bound = next_live_bound(state, d[1], d[0])
            if bound is None:
                bound = CHECKER_BOUND_WHEN_THE_NEXT_LIVE_SIDE_IS_THE_HEAD[variant](state, d)
            if d[1] <= e < bound and state.birth[u] > d[2]:
                return True
        return False

    for (e, u) in sorted(state.dead):
        p = pending(e, u)
        if u in state.head:
            reds.append(f"I-3.5-head-references:{e}:{u}")
        if not (u in live_units or p):
            reds.append(f"I-3.5-reverse:{e}:{u}")
        owning = [side for side in sides if side[0] <= e < side[1]]
        if len(owning) == 1 and not (p or state.birth[u] <= owning[0][0]):
            reds.append(f"I-3.6:{e}:{u}:{owning[0][2]}")
    for u in sorted(live_units - set(state.head)):
        latest = [sn for sn in live if u in sn[3]][-1]
        bound = next_live_bound(state, latest[1], latest[0])
        bound = INF if bound is None else bound
        pins = [(e, v) for (e, v) in state.dead if v == u]
        if not (len(pins) == 1 and latest[1] <= pins[0][0] < bound):
            reds.append(f"I-3.5-exactly-one:{u}")
    return tuple(reds)


def epochs_at_or_above_the_intent(state):
    out = []
    for d in state.snaps:
        if d[4] and next_live_bound(state, d[1], d[0]) is None:
            out += [(e, u) for (e, u) in state.dead if e >= d[5]]
    return out


def successors(state, mode, core_hi, units, limit):
    destroying = [sn[0] for sn in state.snaps if sn[4]]
    if mode == "product" and destroying:
        yield (f"batch{destroying[0]}", destroy_batch(state, destroying[0], core_hi))
        return
    guarded = mode != "unguarded"
    for position in range(units):
        yield (f"overwrite{position}", overwrite(state, position))
    yield ("snapshot", create_snapshot(state, guarded, limit))
    for sn in state.snaps:
        yield (f"intent{sn[0]}", destroy_intent(state, sn[0], guarded))
        yield (f"batch{sn[0]}", destroy_batch(state, sn[0], core_hi))


def explore(mode, core_hi, units, limit, depth):
    start = State(units)
    frontier = {start.key(): (start, ())}
    seen = 0
    tally = {
        "mode": mode, "core_hi": core_hi, "units": units, "snapshot_limit": limit, "depth": depth,
        "states": 0, "states_with_an_open_intent": 0, "release_bugs": 0, "first_release_bug": None,
        "entries_with_epoch_at_or_above_the_intent_txg": 0, "first_epoch_at_or_above_the_intent_txg": None,
        "checker_red_states": {}, "first_checker_red": {}, "first_checker_red_depth": {},
        "verdict_differs_from_infinity": {}, "first_difference": {},
    }
    for variant in CHECKER_BOUND_WHEN_THE_NEXT_LIVE_SIDE_IS_THE_HEAD:
        tally["checker_red_states"][variant] = 0
        if variant != "infinity":
            tally["verdict_differs_from_infinity"][variant] = 0
    for level in range(depth + 1):
        next_frontier = {}
        for key, (state, history) in frontier.items():
            seen += 1
            if any(sn[4] for sn in state.snaps):
                tally["states_with_an_open_intent"] += 1
            if state.bugs:
                tally["release_bugs"] += 1
                if tally["first_release_bug"] is None:
                    tally["first_release_bug"] = {"history": history, "bugs": state.bugs}
            high = epochs_at_or_above_the_intent(state)
            if high:
                tally["entries_with_epoch_at_or_above_the_intent_txg"] += 1
                if tally["first_epoch_at_or_above_the_intent_txg"] is None:
                    tally["first_epoch_at_or_above_the_intent_txg"] = {"history": history, "entries": high}
            verdicts = {v: checker_verdict(state, v) for v in CHECKER_BOUND_WHEN_THE_NEXT_LIVE_SIDE_IS_THE_HEAD}
            for v, reds in verdicts.items():
                if reds:
                    tally["checker_red_states"][v] += 1
                    if v not in tally["first_checker_red"]:
                        tally["first_checker_red"][v] = {"history": history, "reds": reds}
                        tally["first_checker_red_depth"][v] = level
                if v != "infinity" and reds != verdicts["infinity"]:
                    tally["verdict_differs_from_infinity"][v] += 1
                    if v not in tally["first_difference"]:
                        tally["first_difference"][v] = {
                            "history": history, "reds_under_infinity": verdicts["infinity"], "reds_under_it": reds,
                        }
            if level == depth:
                continue
            for (name, child) in successors(state, mode, core_hi, units, limit):
                if child is None:
                    continue
                ck = child.key()
                if ck not in next_frontier:
                    next_frontier[ck] = (child, history + (name,))
        frontier = next_frontier
    tally["states"] = seen
    return tally


def main(argv):
    units = int(argv[1]) if len(argv) > 1 else 2
    limit = int(argv[2]) if len(argv) > 2 else 3
    depth = int(argv[3]) if len(argv) > 3 else 9
    modes = argv[4].split(",") if len(argv) > 4 else ["product", "transaction", "unguarded"]
    for mode in modes:
        for core_hi in CORE_HI_WHEN_THE_NEXT_SIDE_IS_THE_HEAD:
            print(json.dumps(explore(mode, core_hi, units, limit, depth), ensure_ascii=False, sort_keys=True), flush=True)


if __name__ == "__main__":
    main(sys.argv)
