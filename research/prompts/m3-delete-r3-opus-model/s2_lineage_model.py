#!/usr/bin/env python3
"""m3-delete-r3 攻方腿的计数模型（副本外的纯算术模型，被攻过零轮）。

被判对象：S2（正文第一节）的删除 + 墓碑 + 谱系记录形态，按条款字面建模（crates 里这三样都没有实现）：
  意图那次：inode 记录 nlink = 0，deleted_inodes 加号，墓碑树开放容器追加 (号, 出生代, 死亡 txg = 这次的 txg)；
  各批：K = 1，每批去一个单元；最后一批删记录、删队列条目；墓碑不动；
  回退（挂着时一次向前发布）：inode 树、extent、deleted_inodes 取 R_old 的，墓碑树取 cur 的，写一条谱系记录 (T_rb, T_old)；水位取 max；
  崩溃恢复：最新 k 条根读不出，落到最新可读有效根，新实例、写行，新实例第一个根的 txg = 环里读得出的根的最大 txg + 1；
            可写挂载在写行之后把队列走到空（M1）。
谱系记录住处（G4 候选）：
  LC  另一个只增不删的结构，随 cur（S2 原文的读法）；
  LO  同上但随 R_old（取 R_old 那一版再追加这一条）；
  LT  打包记录类型另一个号的容器，挂在墓碑树下、与墓碑容器共用容器号（「同一棵树两种孩子」）。
checker（只看挂载根这一版）：墓碑正推报告 7.2 表的 N1–N5 按字面与按收严的读法各判一次；谱系从挂载根沿父链走：
  有谱系记录的 txg 跳到 T_old，其余 txg 的父取 txg − 1。
历史由动作组成，全部放开枚举（回退之后做什么不写死）：
  mk     建一个空 inode（号 = 水位，出生代 = 这次的 txg）
  nop    一次空发布
  rm:N   删一个活对象：意图 + 各批（同一个会话里连着发）
  cont   把队列走到空（会话里续做；回退进删除中间之后用）
  rb:i   管理员回退到候选集里下标 i 的根（候选：环里读得出、按实例表判有效、不是 cur）
  crash:k 崩溃，最新 k 条根读不出（k = 0 是干净断电），恢复 + 写行 + 续做到空
"""
import os
import sys
from collections import namedtuple, defaultdict

CAP = int(os.environ.get("CAP", "2"))            # 写者的容器容量旋钮（条）
CHECK_CAP = os.environ.get("CHECK_CAP", "knob")   # checker 按什么容量判 N4 / N5：knob（与写者同）或 product（815）
ROOTCAP = int(os.environ.get("ROOTCAP", "0"))     # 墓碑树根装几个孩子；0 = 不限。超过时照 inode 树的写法拒（MoreLeafContainersThanOneRootNodeHolds）
RING = int(os.environ.get("RING", "6"))
FILE_UNITS = int(os.environ.get("FILE_UNITS", "2"))
PRODUCT_CAP = 815

V = namedtuple("V", "txg inst parent objs queue wm tomb lin")
# objs: tuple of (N, B, nlink, units) 按号排；tomb: tuple of (num, ctype, records)；records: tuple of (N, B, d) 或 LT 的 (T_rb, T_old)
# lin: LC / LO 的谱系记录 tuple of (T_rb, T_old)


class Refused(Exception):
    pass


class World:
    __slots__ = ("roots", "readable", "cur", "inst", "rows", "next_txg", "placement", "mutation", "events")

    def __init__(self, placement, mutation):
        self.placement, self.mutation = placement, mutation
        first = V(1, 1, None, ((1, 1, 1, FILE_UNITS),), frozenset(), 2, (), ())
        self.roots = (first,)
        self.readable = (0,)
        self.cur = 0
        self.inst = 1
        self.rows = ()
        self.next_txg = 2
        self.events = ()

    def copy(self):
        w = World.__new__(World)
        for name in World.__slots__:
            setattr(w, name, getattr(self, name))
        return w

    def row(self, inst):
        for (i, t) in self.rows:
            if i == inst:
                return t
        return None

    def valid(self, v):
        t = self.row(v.inst)
        return t is None or v.txg <= t

    def publish(self, objs, queue, wm, tomb, lin, parent=None):
        v = V(self.next_txg, self.inst, self.cur if parent is None else parent, objs, queue, wm, tomb, lin)
        self.next_txg += 1
        self.roots = self.roots + (v,)
        self.readable = self.readable + (len(self.roots) - 1,)
        self.cur = len(self.roots) - 1
        return v

    def curv(self):
        return self.roots[self.cur]

    # ---- 墓碑树的写法（W4：有新记录才重写开放容器，满了开新的；LT 的谱系容器与墓碑容器共用号）
    def append(self, tomb, ctype, record, cap):
        tomb = list(tomb)
        open_index = None
        for index in range(len(tomb) - 1, -1, -1):
            if tomb[index][1] == ctype:
                open_index = index
                break
        if open_index is None or len(tomb[open_index][2]) >= cap:
            if ROOTCAP and len(tomb) + 1 > ROOTCAP:
                raise Refused("tomb_root_full")
            number = 1 + max([c[0] for c in tomb] or [0])
            tomb.append((number, ctype, (record,)))
        else:
            number, kind, records = tomb[open_index]
            tomb[open_index] = (number, kind, records + (record,))
        return tuple(tomb)

    def writer_cap(self):
        return max(1, CAP - 1) if self.mutation == "close_early" and CAP >= 2 else CAP

    # ---- 用户动作
    def op_mk(self):
        v = self.curv()
        objs = v.objs + ((v.wm, self.next_txg, 1, 0),)
        self.publish(objs, v.queue, v.wm + 1, v.tomb, v.lin)

    def op_nop(self):
        v = self.curv()
        self.publish(v.objs, v.queue, v.wm, v.tomb, v.lin)

    def op_rm(self, number):
        v = self.curv()
        txg = self.next_txg
        objs, birth = [], None
        for (n, b, nlink, units) in v.objs:
            if n == number:
                birth = b
                nlink = 1 if self.mutation == "nlink_kept" else 0
            objs.append((n, b, nlink, units))
        tomb = v.tomb
        if self.mutation not in ("no_tomb", "tomb_at_last"):
            death = {"death_minus1": txg - 1, "death_zero": 0}.get(self.mutation, txg)
            # 第 4 段是这条墓碑是哪一次发布写的（根的下标，checker 看不到，只给「真值」用）
            tomb = self.append(tomb, 1, (number, birth, death, len(self.roots)), self.writer_cap())
        self.publish(tuple(objs), v.queue | {number}, v.wm, tomb, v.lin)
        self.continue_queue(in_mount=False)

    def continue_queue(self, in_mount):
        first_in_mount = in_mount
        while self.curv().queue:
            v = self.curv()
            number = min(v.queue)
            entry = [o for o in v.objs if o[0] == number][0]
            n, b, nlink, units = entry
            tomb = v.tomb
            txg = self.next_txg
            if first_in_mount and self.mutation in ("cont_redeath", "cont_dup_tomb"):
                tomb = list(tomb)
                for index, (num, kind, records) in enumerate(tomb):
                    if kind != 1:
                        continue
                    new_records = []
                    for record in records:
                        if record[0] == n and record[1] == b and self.mutation == "cont_redeath":
                            record = (n, b, txg, len(self.roots))
                        new_records.append(record)
                    tomb[index] = (num, kind, tuple(new_records))
                tomb = tuple(tomb)
                if self.mutation == "cont_dup_tomb":
                    tomb = self.append(tomb, 1, (n, b, txg, len(self.roots)), self.writer_cap())
                first_in_mount = False
            if units > 1:
                objs = tuple((o[0], o[1], o[2], o[3] - 1) if o[0] == number else o for o in v.objs)
                self.publish(objs, v.queue, v.wm, tomb, v.lin)
                continue
            if self.mutation == "tomb_at_last":
                tomb = self.append(tomb, 1, (n, b, txg, len(self.roots)), self.writer_cap())
            objs = tuple(o for o in v.objs if o[0] != number)
            self.publish(objs, v.queue - {number}, v.wm, tomb, v.lin)

    def op_cont(self):
        self.continue_queue(in_mount=False)

    def candidates(self):
        ring = self.readable[-RING:]
        return [i for i in ring if i != self.cur and self.valid(self.roots[i])]

    def op_rb(self, target):
        cur, old = self.curv(), self.roots[target]
        t_rb = self.next_txg
        t_old = old.txg - 1 if self.mutation == "lineage_old_minus1" else old.txg
        record = (t_rb, t_old)
        ring_wm = max(self.roots[i].wm for i in self.readable[-RING:])
        wm = max(cur.wm, ring_wm)
        queue = cur.queue if self.mutation == "queue_from_cur" else old.queue
        tomb = old.tomb if self.mutation == "tomb_from_r_old" else cur.tomb
        lin = cur.lin
        if self.mutation != "no_lineage":
            if self.placement == "LC":
                lin = cur.lin + (record,)
            elif self.placement == "LO":
                lin = old.lin + (record,)
            elif self.placement == "LT":
                tomb = self.append(tomb, 5, record, self.writer_cap())
        self.publish(old.objs, queue, wm, tomb, lin, parent=target)

    def op_crash(self, k):
        if k >= len(self.readable):
            return False
        readable = self.readable[:len(self.readable) - k] if k else self.readable
        selected = None
        for index in reversed(readable):
            if self.valid(self.roots[index]):
                selected = index
                break
        if selected is None:
            return False
        self.readable = readable
        v = self.roots[selected]
        self.inst += 1
        rows = dict(self.rows)
        rows[v.inst] = v.txg
        for i in range(v.inst + 1, self.inst):
            rows[i] = 0
        self.rows = tuple(sorted(rows.items()))
        self.cur = selected
        self.next_txg = max(self.roots[i].txg for i in self.readable[-RING:]) + 1
        self.publish(v.objs, v.queue, v.wm, v.tomb, v.lin, parent=selected)
        self.continue_queue(in_mount=True)
        return True

    # ---- checker（只看挂载根这一版；谱系按「有记录跳 T_old、其余 txg − 1」走）
    def lineage_records(self, v):
        if self.placement == "LT":
            return [r for (num, kind, records) in v.tomb if kind == 5 for r in records]
        return list(v.lin)

    def walked_lineage(self, v):
        parent = {}
        for (t_rb, t_old) in self.lineage_records(v):
            parent[t_rb] = t_old
        lineage, t = set(), v.txg
        while t >= 1:
            lineage.add(t)
            t = parent.get(t, t - 1)
        return lineage

    def ancestors(self, index):
        out = set()
        while index is not None:
            out.add(index)
            index = self.roots[index].parent
        return out

    def check(self):
        v = self.curv()
        ccap = CAP if CHECK_CAP == "knob" else PRODUCT_CAP
        objs = {o[0]: o for o in v.objs}
        lineage = self.walked_lineage(v)
        truth = self.ancestors(self.cur)
        red = set()
        kids = list(v.tomb)
        tombs = [r for (num, kind, records) in kids if kind == 1 for r in records]
        if any(kind not in (1, 3) for (_, kind, _) in kids):
            red.add("N1")
        for number in v.queue:
            if number not in objs or objs[number][2] != 0:
                red.add("I95")
            if number not in objs:
                continue
            birth = objs[number][1]
            if not any(r[0] == number and r[1] == birth for r in tombs):
                red.add("N2any")
            if not any(r[0] == number and r[1] == birth and r[2] in lineage for r in tombs):
                red.add("N2lin")
        for (n, b, nlink, units) in v.objs:
            if nlink == 0 and n not in v.queue and units > 0:
                red.add("I47")
        for (n, b, d, writer) in tombs:
            live = n in objs and objs[n][1] == b and objs[n][2] == 1
            if live:
                red.add("N3U")
                if d in lineage:
                    red.add("N3L")
            if n in objs and objs[n][1] != b:
                red.add("I910U")
                if d in lineage:
                    red.add("I910L")
            if (d in lineage) != (writer in truth):
                red.add("lineage_misclassified")
        for (num, kind, records) in kids:
            if kind != 1:
                continue
            keys = [(r[0], r[1]) for r in records]
            if len(set(keys)) != len(keys):
                red.add("N4lit")
            keys3 = [(r[0], r[1], r[2]) for r in records]
            if len(set(keys3)) != len(keys3):
                red.add("N4d")
            keys_on_lineage = [(r[0], r[1]) for r in records if r[2] in lineage]
            if len(set(keys_on_lineage)) != len(keys_on_lineage):
                red.add("N4lin")
            if len(records) > ccap:
                red.add("N4cap")
        numbers = [num for (num, _, _) in kids]
        if numbers and (numbers[0] != 1 or any(a >= b for a, b in zip(numbers, numbers[1:]))):
            red.add("N5order")
        for position, (num, kind, records) in enumerate(kids):
            if kind == 1 and position < len(kids) - 1 and len(records) != ccap:
                red.add("N5lit")
        type1 = [records for (num, kind, records) in kids if kind == 1]
        if any(len(records) != ccap for records in type1[:-1]):
            red.add("N5type")
        if tombs and v.wm <= max(r[0] for r in tombs):
            red.add("I96")
        return red


INVARIANTS = ["N1", "N2any", "N2lin", "N3U", "N3L", "N4lit", "N4d", "N4lin", "N4cap", "N5order", "N5lit", "N5type", "I95", "I47", "I96", "I910U", "I910L", "lineage_misclassified"]


def moves(w):
    v = w.curv()
    out = ["mk", "nop"]
    out += ["rm:%d" % o[0] for o in v.objs if o[2] == 1]
    if v.queue:
        out.append("cont")
    out += ["rb:%d" % i for i in w.candidates()]
    out += ["crash:%d" % k for k in range(0, 4)]
    return out


def apply(w, move):
    w = w.copy()
    try:
        if move == "mk":
            w.op_mk()
        elif move == "nop":
            w.op_nop()
        elif move == "cont":
            w.op_cont()
        elif move.startswith("rm:"):
            w.op_rm(int(move[3:]))
        elif move.startswith("rb:"):
            w.op_rb(int(move[3:]))
        elif move.startswith("crash:"):
            if not w.op_crash(int(move[6:])):
                return None, None
    except Refused as refusal:
        return None, str(refusal)
    return w, None


def enumerate_all(placement, mutation, depth):
    counts = defaultdict(int)
    first = {}
    refusals = defaultdict(int)
    states = 0
    stack = [(World(placement, mutation), ())]
    while stack:
        w, seq = stack.pop()
        if seq:
            states += 1
            for name in w.check():
                counts[name] += 1
                first.setdefault(name, seq)
        if len(seq) < depth:
            for move in moves(w):
                nxt, refused = apply(w, move)
                if refused:
                    refusals[refused] += 1
                    first.setdefault("refused:" + refused, seq + (move,))
                if nxt is not None:
                    stack.append((nxt, seq + (move,)))
    return states, counts, first, refusals


def replay(placement, mutation, seq):
    w = World(placement, mutation)
    for move in seq:
        w, refused = apply(w, move)
        if w is None:
            return None, refused
    return w, None


def logical(w, with_death):
    v = w.curv()
    tombs = sorted((r[0], r[1], r[2]) if with_death else (r[0], r[1]) for (_, kind, records) in v.tomb if kind == 1 for r in records)
    return (tuple(sorted((o[0], o[1], o[2], o[3]) for o in v.objs)), tuple(sorted(v.queue)), tuple(tombs))


def effective_tombs_of(w, number):
    v = w.curv()
    lineage = w.walked_lineage(v)
    return [r[:3] for (_, kind, records) in v.tomb if kind == 1 for r in records if r[0] == number and r[2] in lineage]


def g2(placement):
    """G2：续做在挂载里把墓碑的死亡 txg 改成续做那一次的 txg（或多写一条）——五条判据过不过；之后回退进删除中间再续做，墓碑还在不在谱系上。"""
    for mutation in ["none", "cont_redeath", "cont_dup_tomb"]:
        straight, _ = replay(placement, mutation, ["mk", "rm:1"])
        for lose in range(1, FILE_UNITS + 1):
            crashed, _ = replay(placement, mutation, ["mk", "rm:1", "crash:%d" % lose])
            again, _ = replay(placement, mutation, ["mk", "rm:1", "crash:%d" % lose])
            same_without_death = logical(straight, False) == logical(crashed, False)
            same_with_death = logical(straight, True) == logical(crashed, True)
            plan_equal = logical(crashed, True) == logical(again, True)
            done_writes_nothing = not crashed.curv().queue
            reds = sorted(crashed.check())
            # 回退进删除中间那一版（意图那次的根），在会话里续做到空
            intent_index = [i for i, r in enumerate(crashed.roots) if r.queue and any(o[0] == 1 and o[2] == 0 for o in r.objs)][0]
            after = crashed.copy()
            after.op_rb(intent_index)
            mid_reds = sorted(after.check())
            after.op_cont()
            print(
                "E name=g2 placement=%s mutation=%s lost_roots=%d criterion_3_logical_without_death_txg=%s criterion_3_logical_with_death_txg=%s criterion_1b_same_image_twice=%s criterion_2_done=%s checker_red_after_continuation=%s rollback_to_intent_then_red=%s effective_tombs_of_1_after_rollback_and_continue=%d tombs_of_1_in_tree=%d"
                % (placement, mutation, lose, same_without_death, same_with_death, plan_equal, done_writes_nothing, reds or "-", mid_reds or "-",
                   len(effective_tombs_of(after, 1)), len([r for (_, k, rs) in after.curv().tomb if k == 1 for r in rs if r[0] == 1])))


MUTATIONS = ["none", "no_tomb", "tomb_at_last", "death_minus1", "death_zero", "nlink_kept", "no_lineage", "lineage_old_minus1",
             "tomb_from_r_old", "queue_from_cur", "close_early", "cont_redeath", "cont_dup_tomb"]


def main():
    mode = sys.argv[1]
    if mode == "named":
        named = {
            "delete_rollback_delete_again": ["rm:1", "rb:0", "rm:1"],
            "delete_rollback_before_and_back": ["rm:1", "rb:0", "rb:3"],
            "rollback_into_mid_delete_then_continue": ["rm:1", "rb:1", "cont"],
            "crash_mid_delete_then_rollback_before": ["rm:1", "crash:2", "rb:0"],
            "two_deletes_rollback_between": ["mk", "rm:2", "rm:1", "rb:3", "rm:1"],
        }
        for placement in ["LC", "LO", "LT"]:
            for name, seq in named.items():
                w, refused = replay(placement, "none", seq)
                if w is None:
                    print("E name=named placement=%s history=%s seq=%s result=%s" % (placement, name, ",".join(seq), refused or "illegal"))
                    continue
                v = w.curv()
                print("E name=named placement=%s history=%s seq=%s cap=%d tomb_tree=%s red=%s" % (
                    placement, name, ",".join(seq), CAP, [(num, kind, [r[:3] if kind == 1 else r for r in recs]) for (num, kind, recs) in v.tomb],
                    sorted(w.check()) or "-"))
        return
    if mode == "matrix":
        depth = int(sys.argv[2])
        placements = sys.argv[3].split(",")
        mutations = sys.argv[4].split(",") if len(sys.argv) > 4 else MUTATIONS
        for placement in placements:
            for mutation in mutations:
                states, counts, first, refusals = enumerate_all(placement, mutation, depth)
                cells = " ".join("%s=%d" % (name, counts.get(name, 0)) for name in INVARIANTS)
                firsts = " ".join("first_%s=%s" % (name, ",".join(first[name])) for name in INVARIANTS if name in first)
                refused = " ".join("%s=%d" % (k, n) for k, n in sorted(refusals.items())) or "-"
                print("E name=matrix depth=%d cap=%d check_cap=%s rootcap=%d ring=%d placement=%s mutation=%s states=%d %s refused=%s %s" % (
                    depth, CAP, CHECK_CAP, ROOTCAP, RING, placement, mutation, states, cells, refused, firsts))
        return
    if mode == "g2":
        for placement in sys.argv[2].split(","):
            g2(placement)
        return


if __name__ == "__main__":
    main()
