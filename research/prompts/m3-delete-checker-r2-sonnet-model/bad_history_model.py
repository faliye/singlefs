#!/usr/bin/env python3
"""正推腿：违反「同一条时间线上号不复用」的历史上，C479 各候选能不能红（该红漏判），窗口 K = 候选集只剩最新的 K 条未被抛弃的根。

坏历史 = 在 legal_history_model 的动作之外加一个 press（只供测试的开关把水位压回一个已发过的号，再继续建；
tests/inode_number_watermark_pressed_back_reissues_a_number.rs 的做法）。判「这段历史里确有复用」的口径：
全部未被抛弃的根（不受窗口限制）的记录与墓碑里，同一个号出现两个出生代。
被评估的检查（都只看窗口内的版本）：
  I96     I-9.6 今天的两半：最新根水位 > 窗口内任一版 inode 树内最大 key；水位 > 所选根墓碑最大对象 ID
  jia     I-9.10 按号认、每版比自己的（C479-甲）
  yi      候选集里（记录 + 各版自己谱系上的墓碑）同号只许一个出生代（C479-乙）
  bing    相邻版本（父按谱系配）新号不小于父版水位、共有号出生代相同（C479-丙）
输出：有复用的评估时刻数，其中各检查红了的个数、全都没红的个数。
"""
import random
import sys

import legal_history_model as m


class BadWorld(m.World):
    """写路径先拒中间插入、不拒原地换（inode_number_watermark_pressed_back 用例：压回的号在树里有记录就原地换成新对象；中间没有记录的号拒）：压回的号 < 树内最大 key 且树里没有这个号的记录时拒。"""

    def create(self):
        self.drain()
        cur = self.cur()
        maxkey = max((r[0] for r in cur.records), default=0)
        if cur.wm < maxkey and cur.wm not in {r[0] for r in cur.records}:
            return False
        txg = self.max_txg + 1
        recs = tuple(r for r in cur.records if r[0] != cur.wm) + ((cur.wm, txg, 1),)
        self.publish(records=tuple(sorted(recs)), wm=cur.wm + 1)
        return True


REFUSE_MIDDLE_INSERT = len(sys.argv) > 3 and sys.argv[3] == "refuse"


def press(world):
    cur = world.cur()
    if cur.wm <= 2:
        return False
    new_wm = world.rng.randrange(1, cur.wm)
    world.drain()
    cur = world.cur()
    world.publish(wm=new_wm)
    return True


def reissued(world):
    births = {}
    for i, r in enumerate(world.ring):
        if i in world.abandoned:
            continue
        for n, b, _ in r.records:
            births.setdefault(n, set()).add(b)
        for n, b, _ in r.tombs:
            births.setdefault(n, set()).add(b)
    return any(len(s) > 1 for s in births.values())


def checks(world, window):
    ring = world.ring
    live = [r for i, r in enumerate(ring) if i not in world.abandoned]
    sel = ring[world.newest()]
    cand = sorted(live, key=lambda r: r.txg)[-window:]
    if sel not in cand:
        cand.append(sel)
    out = {}
    maxkey = max((n for v in cand for n, _, _ in v.records), default=0)
    maxtomb = max((n for n, _, _ in sel.tombs), default=0)
    out["I96"] = not (sel.wm > maxkey and sel.wm > maxtomb)
    out["jia"] = any(
        any(n in {a: b for a, b, _ in v.records} and {a: b for a, b, _ in v.records}[n] != b for n, b, _ in v.tombs)
        for v in cand
    )
    births = {}
    for v in cand:
        for n, b, _ in v.records:
            births.setdefault(n, set()).add(b)
        for n, b, d in v.tombs:
            if m.chain_contains(v.txg, v.lineage, d):
                births.setdefault(n, set()).add(b)
    out["yi"] = any(len(s) > 1 for s in births.values())
    by_txg = {v.txg: v for v in cand}
    bing = False
    for v in cand:
        pt = v.txg - 1
        for rb, old in v.lineage:
            if rb == v.txg:
                pt = old
        p = by_txg.get(pt)
        if p is None:
            continue
        pn = {n: b for n, b, _ in p.records}
        if any((n not in pn and n < p.wm) or (n in pn and pn[n] != b) for n, b, _ in v.records):
            bing = True
    out["bing"] = bing
    return out


def run(seeds, length, window):
    total = reissue_cnt = 0
    hit = {"I96": 0, "jia": 0, "yi": 0, "bing": 0}
    none_hit = 0
    legal_red = {"I96": 0, "jia": 0, "yi": 0, "bing": 0}
    for seed in range(seeds):
        rng = random.Random(seed)
        w = BadWorld(rng) if REFUSE_MIDDLE_INSERT else m.World(rng)
        pressed = False
        for _ in range(length):
            op = rng.choice(["create", "create", "unlink", "unlink", "rollback", "crash", "press"])
            if op == "press":
                ok = press(w)
                pressed = pressed or bool(ok)
            else:
                ok = {"create": w.create, "unlink": w.unlink, "rollback": w.rollback, "crash": w.crash_then_publish}[op]()
            if ok is False:
                continue
            total += 1
            c = checks(w, window)
            if reissued(w):
                reissue_cnt += 1
                for k, v in c.items():
                    hit[k] += int(v)
                if not any(c.values()):
                    none_hit += 1
            else:
                for k, v in c.items():
                    legal_red[k] += int(v)
    return total, reissue_cnt, hit, none_hit, legal_red


if __name__ == "__main__":
    seeds = int(sys.argv[1]) if len(sys.argv) > 1 else 3000
    length = int(sys.argv[2]) if len(sys.argv) > 2 else 10
    for window in (3, 6, 1000):
        total, rc, hit, nh, lr = run(seeds, length, window)
        print(f"窗口 K={window}: 评估时刻 {total}，有复用 {rc}；红了 {hit}；全没红 {nh}；没有复用却红了 {lr}")
