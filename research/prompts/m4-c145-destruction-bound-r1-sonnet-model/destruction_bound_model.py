#!/usr/bin/env python3
"""销毁意图区间上界的小模型（推论轮 m4-c145-destruction-bound-r1 正推腿，独立于 crates/ 的代码，只抄条款的语义）。

状态 = 活头的 previous_snapshot_txg、快照链（txg、prev、销毁中记号）、活头单元（出生 txg）、deadlist 条目（纪元、出生 txg、单元号）。
操作 = 覆盖写（杀一个活头单元，出生 <= 头的前驱则进 deadlist、纪元 = 头的前驱，否则直接释放）、建快照、写销毁意图、放一批、放最后一批。
崩溃与续做在模型里没有状态：hi 不落盘，状态全在盘上那几样里（所以不另设操作）。
每一版（每个操作之后的状态）上判：
  相同性：下一侧是头时，三种待放集（上界 = 无穷 / 写意图那次的 txg / 本版的 txg）在每一条盘上条目上结论是否相同；
  检查器判定：I-3.5 的 ④、逆向、③ 与 I-3.6（照 crates/singlefs-checker/src/walk.rs 的 judge_the_deadlist_against_the_head_and_the_live_snapshots 的语义，上界取无穷）。
注意：模型里的“检查器判定”只有 ③ ④ 逆向与 I-3.6，没有写 I-9.25（树表条目 previous_snapshot_txg 取值）的前驱链那一半；真 checker 有它，
所以“写意图不传前驱”那个变异在真 checker 里于意图那一版就红（见 probe/ 的控制实验），模型里的数字不代表真 checker 抓到的时点。
用法：python3 destruction_bound_model.py [--mutant 名字] [--depth N]
"""
import argparse
import itertools
import json
import sys

INF = float("inf")
MAX_SNAPSHOTS = 3


class State:
    __slots__ = ("txg", "head_prev", "snaps", "units", "dead", "killed_at", "log")

    def __init__(self):
        self.txg = 1                    # 第一个文件版的 txg
        self.head_prev = 0
        self.snaps = []                 # 每项 [txg, prev, destroying, intent_txg]，按 txg 升序
        self.units = {1: 1}             # 单元号 -> 出生 txg（活头引用的）
        self.dead = []                  # deadlist 条目 (epoch, birth, uid)，按键序
        self.killed_at = {}             # 单元号 -> 被杀的 txg（活快照引用判定要用）
        self.log = []

    def key(self):
        return (self.txg, self.head_prev, tuple(tuple(s) for s in self.snaps),
                tuple(sorted(self.units.items())), tuple(self.dead), tuple(sorted(self.killed_at.items())))

    def copy(self):
        n = State.__new__(State)
        n.txg = self.txg
        n.head_prev = self.head_prev
        n.snaps = [list(s) for s in self.snaps]
        n.units = dict(self.units)
        n.dead = list(self.dead)
        n.killed_at = dict(self.killed_at)
        n.log = list(self.log)
        return n


def next_side(st, idx):
    return ("snap", idx + 1) if idx + 1 < len(st.snaps) else ("head", None)


def pending_set_hi(st, entry, mode):
    """条目在不在某个未完成销毁意图的待放集里；下一侧是头时上界按 mode 取（inf / intent / version）。"""
    epoch, birth, _ = entry
    for i, (stxg, prev, destroying, intent_txg) in enumerate(st.snaps):
        if not destroying:
            continue
        kind, j = next_side(st, i)
        if kind == "snap":
            hi = st.snaps[j][0]
        elif mode == "inf":
            hi = INF
        elif mode == "intent":
            hi = intent_txg
        else:
            hi = st.txg
        if stxg <= epoch < hi and birth > prev:
            return True
    return False


def live_snapshots(st):
    return [s for s in st.snaps if not s[2]]


def references(snap_txg, uid, st, birth_of):
    return birth_of[uid] <= snap_txg and st.killed_at.get(uid, INF) > snap_txg


def judge(st, birth_of, mode="inf"):
    """检查器的 I-3.5 ③ ④ 逆向与 I-3.6（walk.rs 同名判定的语义）。返回红的描述列表。"""
    reds = []
    live = live_snapshots(st)
    head_units = set(st.units)
    # 一侧：快照 X 的 [prev, X.txg)，头的 [head_prev, inf)
    sides = [(s[1], s[0]) for s in live] + [(st.head_prev, INF)]
    ref_by_snap = {s[0]: {u for u in birth_of if references(s[0], u, st, birth_of)} for s in live}
    for e in st.dead:
        epoch, birth, uid = e
        pend = pending_set_hi(st, e, mode)
        if uid in head_units:
            reds.append(("④", e))
        if not (any(uid in ref_by_snap[s[0]] for s in live) or pend):
            reds.append(("逆向", e))
        owning = [sd for sd in sides if sd[0] <= epoch < sd[1]]
        if len(owning) == 1:
            if not (pend or birth <= owning[0][0]):
                reds.append(("I-3.6", e))
    for uid in {u for ru in ref_by_snap.values() for u in ru} - head_units:
        latest = [s for s in live if uid in ref_by_snap[s[0]]][-1]
        nxt = [s for s in live if (s[0]) > latest[0]]
        ub = nxt[0][0] if nxt else INF
        pin = [e for e in st.dead if e[2] == uid]
        if not (len(pin) == 1 and latest[0] <= pin[0][0] < ub):
            reds.append(("③", (uid, pin)))
    return reds


def released_by_batch(st, i, mode, batch_txg):
    stxg, prev, _, intent_txg = st.snaps[i]
    kind, j = next_side(st, i)
    if kind == "snap":
        hi = st.snaps[j][0]
    elif mode == "inf":
        hi = INF
    elif mode == "intent":
        hi = intent_txg
    elif mode == "batch":
        hi = batch_txg
    elif mode == "latest-published":
        hi = batch_txg - 1
    else:
        raise ValueError(mode)
    return sorted(e for e in st.dead if stxg <= e[0] < hi and e[1] > prev)


def apply(st, op, M):
    """施加一个操作；返回新状态，或 None（被拒，状态不变）。M 是变异开关集合。"""
    kind = op[0]
    n = st.copy()
    n.log.append(op)
    if kind == "W":
        t = n.txg + 1
        uid_old = op[1]
        if uid_old not in n.units:
            return None
        birth = n.units.pop(uid_old)
        n.killed_at[uid_old] = t
        if birth <= n.head_prev:
            n.dead.append((n.head_prev, birth, uid_old))
            n.dead.sort()
        n.units[max(list(n.units) + list(n.killed_at) + [1]) + 1] = t
        n.txg = t
        return n
    if kind == "C":
        if any(s[2] for s in n.snaps) and "allow-create-while-destroying" not in M:
            return None
        if len(n.snaps) >= MAX_SNAPSHOTS:
            return None
        t = n.txg + 1
        n.snaps.append([t, n.head_prev, False, 0])
        n.head_prev = t
        n.txg = t
        return n
    if kind == "I":
        i = op[1]
        if i >= len(n.snaps) or n.snaps[i][2]:
            return None
        if any(s[2] for s in n.snaps) and "allow-two-destructions" not in M:
            return None
        t = n.txg + 1
        n.snaps[i][2] = True
        n.snaps[i][3] = t
        if "intent-does-not-pass-the-predecessor" not in M:
            k, j = next_side(n, i)
            if k == "head":
                n.head_prev = n.snaps[i][1]
            else:
                n.snaps[j][1] = n.snaps[i][1]
        n.txg = t
        return n
    if kind in ("B", "F"):
        i = op[1]
        if i >= len(n.snaps) or not n.snaps[i][2]:
            return None
        t = n.txg + 1
        mode = "latest-published" if "hi-is-latest-published" in M else "batch"
        rel = released_by_batch(n, i, mode, t)
        take = rel if kind == "F" else rel[:op[2]]
        for e in take:
            n.dead.remove(e)
        if kind == "F":
            del n.snaps[i]
        n.txg = t
        return n
    raise ValueError(op)


def operations(st):
    ops = []
    for uid in sorted(st.units):
        ops.append(("W", uid))
    ops.append(("C",))
    for i in range(len(st.snaps)):
        if not st.snaps[i][2]:
            ops.append(("I", i))
        else:
            ops.append(("B", i, 1))
            ops.append(("F", i))
    return ops


def explore(depth, M):
    seen = {}
    frontier = [State()]
    seen[frontier[0].key()] = 0
    stats = {"states": 0, "flagged_head_next_states": 0, "entries_in_such_states": 0,
             "pending_differs": [], "release_differs": [], "judge_reds": [], "batches_checked": 0,
             "delta_reds": [], "transitions_checked": 0, "delta_reds_child_single_version_judge_green": 0}
    birth_of_cache = {}
    for d in range(depth + 1):
        nxt = []
        for st in frontier:
            stats["states"] += 1
            birth_of = {}
            for uid, b in st.units.items():
                birth_of[uid] = b
            for uid in st.killed_at:
                birth_of.setdefault(uid, None)
            # 被杀单元的出生 txg 要保留：从日志重放得到
            birth_of = births_by_replay(st)
            flagged_head = [i for i, s in enumerate(st.snaps) if s[2] and next_side(st, i)[0] == "head"]
            if flagged_head:
                stats["flagged_head_next_states"] += 1
                stats["entries_in_such_states"] += len(st.dead)
                for e in st.dead:
                    a = pending_set_hi(st, e, "inf")
                    b = pending_set_hi(st, e, "intent")
                    c = pending_set_hi(st, e, "version")
                    if not (a == b == c):
                        stats["pending_differs"].append((st.log, e, a, b, c))
            for i, s in enumerate(st.snaps):
                if s[2]:
                    t = st.txg + 1
                    sets = {m: released_by_batch(st, i, m, t) for m in ("inf", "intent", "batch")}
                    stats["batches_checked"] += 1
                    if not (sets["inf"] == sets["intent"] == sets["batch"]):
                        stats["release_differs"].append((st.log, i, sets))
            reds = judge(st, birth_of)
            if reds:
                stats["judge_reds"].append((st.log, reds))
            if d == depth:
                continue
            for op in operations(st):
                n = apply(st, op, M)
                if n is None:
                    continue
                # 跨版本判（不改格式的替代判法）：相邻两版里 D 在前一版已带销毁中记号、下一侧是头，则后一版新出现的条目纪元必须 < D.S.txg
                # （写意图之后的发布杀单元，纪元 = 头的前驱 = P < lo）。
                stats["transitions_checked"] += 1
                fresh = set(n.dead) - set(st.dead)
                for i, s_ in enumerate(st.snaps):
                    if s_[2] and next_side(st, i)[0] == "head" and i < len(n.snaps) and n.snaps[i][0] == s_[0] and n.snaps[i][2]:
                        for e in fresh:
                            if e[0] >= s_[0]:
                                stats["delta_reds"].append((n.log, e))
                                if not judge(n, births_by_replay(n)):
                                    stats["delta_reds_child_single_version_judge_green"] += 1
                k = n.key()
                if k in seen:
                    continue
                seen[k] = d + 1
                nxt.append(n)
        frontier = nxt
    return stats


def births_by_replay(st):
    """从日志重放出每个单元号的出生 txg（被杀的单元不在 units 里了）。"""
    births = {1: 1}
    txg = 1
    for op in st.log:
        txg += 1
        if op[0] == "W":
            nid = max(births) + 1
            births[nid] = txg
    return births


def inject_stray_epoch(depth, M):
    """校验路径自证：在一个“销毁中且下一侧是头”的状态里手塞一条纪元 >= 本版 txg 的条目，看三种待放集谁放过它。"""
    out = []
    seen = set()
    frontier = [State()]
    for d in range(depth + 1):
        nxt = []
        for st in frontier:
            if any(s[2] and next_side(st, i)[0] == "head" for i, s in enumerate(st.snaps)):
                forged = (st.txg + 5, st.snaps[[i for i, s in enumerate(st.snaps) if s[2]][0]][1] + 1, 999)
                row = {m: pending_set_hi(_with(st, forged), forged, m) for m in ("inf", "intent", "version")}
                out.append(row)
            if d == depth:
                continue
            for op in operations(st):
                n = apply(st, op, M)
                if n is None or n.key() in seen:
                    continue
                seen.add(n.key())
                nxt.append(n)
        frontier = nxt
    return out


def _with(st, entry):
    n = st.copy()
    n.dead.append(entry)
    return n


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--mutant", default="")
    ap.add_argument("--depth", type=int, default=7)
    ap.add_argument("--stray", action="store_true")
    a = ap.parse_args()
    M = {m for m in a.mutant.split(",") if m}
    if a.stray:
        rows = inject_stray_epoch(a.depth, M)
        agg = {}
        for r in rows:
            k = tuple(sorted(r.items()))
            agg[k] = agg.get(k, 0) + 1
        for k, v in sorted(agg.items()):
            print("stray-epoch-entry pending(inf/intent/version) =", dict(k), "states =", v)
        return 0
    s = explore(a.depth, M)
    print(f"mutant={a.mutant or '-'} depth={a.depth} states={s['states']} "
          f"flagged_head_next_states={s['flagged_head_next_states']} entries_in_such_states={s['entries_in_such_states']} "
          f"batches_checked={s['batches_checked']}")
    print(f"pending_set_differs={len(s['pending_differs'])} release_set_differs={len(s['release_differs'])} judge_reds={len(s['judge_reds'])} "
          f"cross_version_new_entry_reaches_lo={len(s['delta_reds'])} (transitions_checked={s['transitions_checked']}, "
          f"child_version_single_version_judge_green={s['delta_reds_child_single_version_judge_green']})")
    for k in ("pending_differs", "release_differs", "judge_reds", "delta_reds"):
        for item in s[k][:2]:
            print(" ", k, json.dumps(item, default=str, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    sys.exit(main())
