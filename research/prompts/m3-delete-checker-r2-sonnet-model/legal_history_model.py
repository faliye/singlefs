#!/usr/bin/env python3
"""删除线 checker 并进三·3：各候选读法在「合法历史」上会不会误红的计数模型（正推腿，纯算术，不读盘、不跑 crates）。

模型的前提只来自条款与 crates 今天的实现（逐条出处在报告里）：
- 每次发布 txg 加一；崩溃恢复落到较旧根 R_k 之后，新实例第一个根的 txg = 根环里最大 txg + 1
  （.claude/kb/checks-owed.md C616 行）；候选集按最新根的实例表滤掉被抛弃的根（walk.rs abandoned_by_the_newest_roots_table）。
- 意图那次发布：记录 nlink 0，deleted_inodes 加号，墓碑树追加 (号, 出生代, 整对象, 本次 txg)；最后一批：删记录、删队列项（D8 已定项 16）。
- 管理员回退：新根取 R_old 的 inode 树 / deleted_inodes，墓碑树与谱系树取现行，谱系追加 (T_rb, T_old)，水位取 max
  （singlefs-core/src/mount.rs version_to_publish_the_rollback_after；D8 已定项 6 第 171 行）。
- 水位只增，崩溃恢复落到 R_k 后水位随 R_k 载入（D8 已定项 6 第 173 行）。会话里每次用户改动先把队列走空（D8 已定项 16 第 6 条）。
不模拟分批、不模拟根环容量与回退下界 F（候选集取全部未被抛弃的根）。

读法（被评估的候选）：
  T2a_ver   I-9.10 墓碑按号认、每版比自己的记录
  T2a_x     I-9.10 墓碑按号认、所选根墓碑去比候选集里任一版的记录（跨版本）
  T3a_*     I-9.18 / I-9.19 / I-9.20 每版用自己的 txg 与自己的谱系树起算「在谱系上」
  T3b_*     一律用所选根的 txg 与谱系树
  N4lit     I-9.20 不按谱系过滤（对照）
  C479_yi   候选集里（记录 + 各版自己谱系上的墓碑）同号只许一个出生代
  C479_bing 相邻版本（父 = 谱系给的 T_old，否则 txg-1 那一版，都在候选集里时）新号不小于父版水位、共有号出生代相同
  T12       I-9.6：最新根水位 > 候选集任一版 inode 树内最大 key 与所选根墓碑最大对象 ID
输出：每条读法在多少个（历史前缀）上误红。合法历史上误红应为 0；非 0 的读法出局。
"""
import random
import sys
from dataclasses import dataclass


@dataclass(frozen=True)
class Root:
    txg: int
    inst: int
    records: tuple  # ((号, 出生代, nlink), ...)
    queue: frozenset
    tombs: tuple  # ((号, 出生代, 死亡 txg), ...)
    lineage: tuple  # ((T_rb, T_old), ...)
    wm: int


def chain_contains(txg_of_version, lineage, death):
    """在谱系上：从 txg_of_version 起往回走，有谱系记录 (T_rb, T_old) 的 txg 的父是 T_old，其余 txg 的父是 txg-1。"""
    cur = txg_of_version
    for rb, old in sorted(lineage, reverse=True):
        if rb > cur:
            continue
        if rb <= death <= cur:
            return True
        cur = old
    return 0 <= death <= cur


class World:
    def __init__(self, rng):
        self.rng = rng
        self.ring = [Root(1, 1, (), frozenset(), (), (), 1)]
        self.abandoned = set()
        self.inst = 1
        self.max_txg = 1

    def newest(self):
        return max(range(len(self.ring)), key=lambda i: (self.ring[i].txg, self.ring[i].inst))

    def cur(self):
        return self.ring[self.newest()]

    def publish(self, **kw):
        cur = self.cur()
        txg = self.max_txg + 1
        new = Root(txg, cur.inst, kw.get("records", cur.records), kw.get("queue", cur.queue),
                   kw.get("tombs", cur.tombs), kw.get("lineage", cur.lineage), kw.get("wm", cur.wm))
        self.ring.append(new)
        self.max_txg = txg

    def drain(self):
        for n in sorted(self.cur().queue):
            cur = self.cur()
            self.publish(records=tuple(r for r in cur.records if r[0] != n), queue=cur.queue - {n})

    def create(self):
        self.drain()
        cur = self.cur()
        self.publish(records=cur.records + ((cur.wm, self.max_txg + 1, 1),), wm=cur.wm + 1)

    def unlink(self):
        self.drain()
        cur = self.cur()
        alive = [r for r in cur.records if r[2] >= 1]
        if not alive:
            return False
        n, birth, _ = self.rng.choice(alive)
        txg = self.max_txg + 1
        recs = tuple((a, b, 0) if a == n else (a, b, c) for a, b, c in cur.records)
        self.publish(records=recs, queue=cur.queue | {n}, tombs=cur.tombs + ((n, birth, txg),))
        if self.rng.random() < 0.6:
            self.drain()
        return True

    def rollback(self):
        cur = self.cur()
        cands = [i for i in range(len(self.ring)) if i not in self.abandoned and self.ring[i].txg < cur.txg]
        if not cands:
            return False
        old = self.ring[self.rng.choice(cands)]
        txg = self.max_txg + 1
        self.publish(records=old.records, queue=old.queue, lineage=cur.lineage + ((txg, old.txg),),
                     wm=max(cur.wm, old.wm))
        return True

    def crash_then_publish(self):
        live = [i for i in range(len(self.ring)) if i not in self.abandoned]
        newest = self.newest()
        cands = [i for i in live if i != newest]
        if not cands:
            return False
        rk = self.ring[self.rng.choice(cands)]
        for i in live:
            r = self.ring[i]
            if r.txg > rk.txg:
                self.abandoned.add(i)
        self.inst += 1
        self.ring.append(Root(self.max_txg + 1, self.inst, rk.records, rk.queue, rk.tombs, rk.lineage, rk.wm))
        self.max_txg += 1
        return True


def evaluate(world):
    ring = world.ring
    sel = ring[world.newest()]
    cand = [r for i, r in enumerate(ring) if i not in world.abandoned]
    red = {}

    def note(name):
        red[name] = red.get(name, 0) + 1

    for v in cand:
        rec = {n: b for n, b, _ in v.records}
        if any(n in rec and rec[n] != b for n, b, _ in v.tombs):
            note("T2a_ver")
            break
    allrec = {}
    for v in cand:
        for n, b, _ in v.records:
            allrec.setdefault(n, set()).add(b)
    if any(n in allrec and allrec[n] != {b} for n, b, _ in sel.tombs):
        note("T2a_x")
    for tag, pick in (("T3a", lambda v: (v.txg, v.lineage)), ("T3b", lambda v: (sel.txg, sel.lineage))):
        bad = {"I9.18": False, "I9.19": False, "I9.20": False}
        for v in cand:
            txg, lin = pick(v)
            rec = {n: (b, c) for n, b, c in v.records}
            on = [t for t in v.tombs if chain_contains(txg, lin, t[2])]
            for n in v.queue:
                if n in rec and not any(t[0] == n and t[1] == rec[n][0] for t in on):
                    bad["I9.18"] = True
            for n, b, _ in on:
                if n in rec and rec[n][0] == b and rec[n][1] != 0:
                    bad["I9.19"] = True
            keys = [(t[0], t[1]) for t in on]
            if len(keys) != len(set(keys)):
                bad["I9.20"] = True
        for k, flag in bad.items():
            if flag:
                note(f"{tag}_{k}")
    for v in cand:
        keys = [(t[0], t[1]) for t in v.tombs]
        if len(keys) != len(set(keys)):
            note("N4lit_I9.20")
            break
    births = {}
    for v in cand:
        for n, b, _ in v.records:
            births.setdefault(n, set()).add(b)
        for n, b, d in v.tombs:
            if chain_contains(v.txg, v.lineage, d):
                births.setdefault(n, set()).add(b)
    if any(len(s) > 1 for s in births.values()):
        note("C479_yi")
    by_txg = {v.txg: v for v in cand}
    for v in cand:
        parent_txg = v.txg - 1
        for rb, old in v.lineage:
            if rb == v.txg:
                parent_txg = old
        p = by_txg.get(parent_txg)
        if p is None:
            continue
        pn = {n: b for n, b, _ in p.records}
        if any((n not in pn and n < p.wm) or (n in pn and pn[n] != b) for n, b, _ in v.records):
            note("C479_bing")
            break
    maxkey = max((n for v in cand for n, _, _ in v.records), default=0)
    maxtomb = max((n for n, _, _ in sel.tombs), default=0)
    if not (sel.wm > maxkey and sel.wm > maxtomb):
        note("T12")
    return red


def run(seeds, length):
    total, first, evaluated = {}, {}, 0
    for seed in range(seeds):
        rng = random.Random(seed)
        w = World(rng)
        ops = []
        for _ in range(length):
            op = rng.choice(["create", "create", "unlink", "unlink", "rollback", "crash"])
            ok = {"create": w.create, "unlink": w.unlink, "rollback": w.rollback, "crash": w.crash_then_publish}[op]()
            if ok is False:
                continue
            ops.append(op)
            evaluated += 1
            for k, c in evaluate(w).items():
                total[k] = total.get(k, 0) + 1
                if k not in first or len(ops) < len(first[k][1]):
                    first[k] = (seed, list(ops))
    return evaluated, total, first


if __name__ == "__main__":
    seeds = int(sys.argv[1]) if len(sys.argv) > 1 else 3000
    length = int(sys.argv[2]) if len(sys.argv) > 2 else 10
    n, total, first = run(seeds, length)
    print(f"评估时刻 {n}（seeds={seeds}, length={length}）")
    names = ["T2a_ver", "T2a_x", "T3a_I9.18", "T3a_I9.19", "T3a_I9.20", "T3b_I9.18", "T3b_I9.19", "T3b_I9.20",
             "N4lit_I9.20", "C479_yi", "C479_bing", "T12"]
    for k in names:
        print(f"误红 {k} = {total.get(k, 0)}")
    for k in names:
        if k in first:
            print(f"最短反例 {k}: seed={first[k][0]} ops={first[k][1]}")
