#!/usr/bin/env python3
"""m3-tomb-r1 攻方腿的计数模型（副本外的纯算术模型，被攻过零轮）。

按条款推的扫盘重建（C113 定案 P2 已发布谓词 + P3 同身份取择新键最大者；crates 里没有实现）在合法历史上
判「已删对象判活」(DJA) 与「活对象判死」(LJD) 的次数，比较墓碑载体在管理员回退 / 崩溃恢复落到旧根 / 号重发时的几种处置（臂）。

历史由用户动作组成，全部放开枚举（不写死回退之后做什么）：
  mk        建一个对象（号 = 水位，出生代 = 这次发布的 txg）
  rm:N      删掉一个活对象：意图那次发布写一条墓碑（删除本体的分批与本模型无关：墓碑只在意图那次写）
  nop       一次不写墓碑的发布
  rb:T      管理员回退到候选集里 txg = T 的根（挂着时的一次向前发布，不写实例表行，D23 已定项 14）
  crash:k   崩溃，最新 k 条根读不出（连同它们的 journal 记录），恢复落到最新可读的根：新实例、写行 (i, T_sel)、
            新实例第一个根的 txg = 环里读得出的根的最大 txg + 1（D23 已定项 14 注 3），水位随所选根（号会重发）
墓碑容器每个装 CAP 条记录（取小值以触发换容器）。

臂（墓碑容器在回退那次发布怎么处置；扫描怎么判）：
  cur         容器索引照现行那一版（D23 已定项 14 里树表、分配记录等「现算 / 照 cur」那一类的默认），记录 = (N, B)
  old         容器索引取 R_old 那一版（像 inode、extent 两棵），不重写容器，记录 = (N, B)
  old_rw      取 R_old 那一版，且回退那次发布把内容与 R_old 不同的容器重写成 R_old 的内容（R_old 里没有的写一个空版本），
              空版本留在索引里（有结构指着），记录 = (N, B)
  old_rw_unref 同 old_rw，但空版本不留在索引里（没有结构指着、是可被复用的垃圾）；扫描时按最坏情形假设它们已被覆盖
  cur_int     照 cur，记录 = (N, B, 死亡 txg)；回退那次发布追加一条区间记录 (T_old, T_rb]，扫描把死亡 txg 落在任一区间里的墓碑作废
  cur_lin     照 cur，记录 = (N, B, 死亡 txg)；回退那次发布追加一条谱系记录 (T_rb 的父 = T_old)，扫描从挂载根沿父链走出谱系，
              只认死亡 txg 在谱系上的墓碑
另有一行共用前提的对照 inode_leaf：inode 叶（打包记录类型 2）按今天的代码回退时取 R_old 那一版、不重写，扫描按 P2 / P3 择版，
看扫出来的 nlink 状态与真值差几次（与墓碑无关，判「打中分不分辨臂」用）。
"""
import itertools
import random
import sys
from collections import defaultdict

import os
CAP = int(os.environ.get("TOMB_CAP", "2"))
RING = int(os.environ.get("TOMB_RING", "6"))
ARMS = ["cur", "old", "old_rw", "old_rw_unref", "cur_int", "cur_lin"]


class Root:
    __slots__ = ("txg", "inst", "parent", "objs", "wm", "tomb", "meta", "leaf_v", "is_rb", "rb_old")

    def __init__(self, txg, inst, parent, objs, wm, tomb, meta, leaf_v, is_rb=False, rb_old=None):
        self.txg, self.inst, self.parent = txg, inst, parent
        self.objs = objs          # {(N, B): 'live' | 'dead'}
        self.wm = wm
        self.tomb = tomb          # arm -> (view: {cid: tuple(records)}, open_cid or None)
        self.meta = meta          # arm -> tuple(meta records)
        self.leaf_v = leaf_v      # 引用的 inode 叶版本号（disk 里 leaf 版本的下标）
        self.is_rb, self.rb_old = is_rb, rb_old


class World:
    def __init__(self):
        self.seq = 0
        self.inst = 1
        self.rows = {}            # inst -> T_pub
        self.roots = []           # 全部发过的根（含读不出的）
        self.readable = []        # 可读根的下标，按发布次序
        # 盘：每臂一份码 3 墓碑版本表；inode 叶一份
        self.disk = {a: [] for a in ARMS}       # (cid, birth, inst, seq, records, unref_empty)
        self.meta_disk = {a: [] for a in ARMS}  # (birth, inst, seq, meta_records)
        self.leaf_disk = []                     # (birth, inst, seq, objs_snapshot)
        objs = {}
        tomb = {a: ({}, None) for a in ARMS}
        meta = {a: () for a in ARMS}
        self.leaf_disk.append((1, 1, 0, {}))
        r = Root(1, 1, None, objs, 1, tomb, meta, 0)
        self.roots.append(r)
        self.readable.append(0)
        self.cur = 0
        self.next_txg = 2

    # ---- 已发布谓词（D18 已定项 11「已发布谓词（全局）」码 3 那一支）
    def published(self, birth, inst):
        mount = self.roots[self.cur]
        if inst < mount.inst:
            if inst in self.rows:
                return birth <= self.rows[inst]
            return True
        if inst == mount.inst:
            return birth <= mount.txg
        return False

    def ring(self):
        return self.readable[-RING:]

    def candidates(self):
        out = []
        for idx in self.ring():
            r = self.roots[idx]
            if idx == self.cur:
                continue
            if r.inst in self.rows and r.txg > self.rows[r.inst]:
                continue
            out.append(idx)
        return out

    # ---- 写一次发布
    def publish(self, objs, wm, tomb, meta, new_tomb_versions, new_meta_versions, leaf_changed, is_rb=False, rb_old=None, parent=None, leaf_v=None):
        txg = self.next_txg
        self.next_txg += 1
        self.seq += 1
        for arm, versions in new_tomb_versions.items():
            for (cid, recs, unref) in versions:
                self.disk[arm].append((cid, txg, self.inst, self.seq, recs, unref))
        for arm, mrec in new_meta_versions.items():
            self.meta_disk[arm].append((txg, self.inst, self.seq, mrec))
        if leaf_changed:
            self.leaf_disk.append((txg, self.inst, self.seq, dict(objs)))
            leaf_v = len(self.leaf_disk) - 1
        r = Root(txg, self.inst, self.cur if parent is None else parent, objs, wm, tomb, meta, leaf_v, is_rb, rb_old)
        self.roots.append(r)
        idx = len(self.roots) - 1
        self.readable.append(idx)
        self.cur = idx
        return r

    def append_tomb(self, arm, view_open, record):
        view, open_cid = view_open
        view = dict(view)
        if open_cid is None or len(view[open_cid]) >= CAP:
            num = 1 + max([c[0] for c in view] or [0])
            open_cid = (num, self.next_txg)
            view[open_cid] = ()
        view[open_cid] = view[open_cid] + (record,)
        return (view, open_cid), [(open_cid, view[open_cid], False)]

    # ---- 用户动作
    def op_mk(self):
        r = self.roots[self.cur]
        objs = dict(r.objs)
        objs[(r.wm, self.next_txg)] = "live"
        self.publish(objs, r.wm + 1, r.tomb, r.meta, {}, {}, True)

    def op_nop(self):
        r = self.roots[self.cur]
        self.publish(dict(r.objs), r.wm, r.tomb, r.meta, {}, {}, False, leaf_v=r.leaf_v)

    def op_rm(self, key):
        r = self.roots[self.cur]
        objs = dict(r.objs)
        objs[key] = "dead"
        death = self.next_txg
        tomb, newv = {}, {}
        for arm in ARMS:
            rec = key if arm in ("cur", "old", "old_rw", "old_rw_unref") else (key[0], key[1], death)
            tomb[arm], newv[arm] = self.append_tomb(arm, r.tomb[arm], rec)
        self.publish(objs, r.wm, tomb, r.meta, newv, {}, True)

    def op_rb(self, target):
        cur = self.roots[self.cur]
        old = self.roots[target]
        t_rb = self.next_txg
        tomb, newv, meta, newm = {}, {}, {}, {}
        for arm in ARMS:
            if arm == "cur":
                tomb[arm] = cur.tomb[arm]
                meta[arm] = cur.meta[arm]
            elif arm == "old":
                tomb[arm] = old.tomb[arm]
                meta[arm] = cur.meta[arm]
            elif arm in ("old_rw", "old_rw_unref"):
                cview, _ = cur.tomb[arm]
                oview, oopen = old.tomb[arm]
                view = dict(oview)
                writes = []
                for cid in set(cview) | set(oview):
                    want = oview.get(cid, ())
                    if cview.get(cid, ()) != want or (cid not in oview):
                        unref = cid not in oview and arm == "old_rw_unref"
                        writes.append((cid, want, unref))
                        if cid not in oview and arm == "old_rw":
                            view[cid] = ()
                tomb[arm] = (view, oopen)
                newv[arm] = writes
                meta[arm] = cur.meta[arm]
            elif arm == "cur_int":
                tomb[arm] = cur.tomb[arm]
                meta[arm] = cur.meta[arm] + (("int", old.txg, t_rb),)
                newm[arm] = meta[arm]
            elif arm == "cur_lin":
                tomb[arm] = cur.tomb[arm]
                meta[arm] = cur.meta[arm] + (("lin", t_rb, old.txg),)
                newm[arm] = meta[arm]
        wm = max(cur.wm, max(self.roots[i].wm for i in self.ring()))
        self.publish(dict(old.objs), wm, tomb, meta, newv, newm, False, is_rb=True, rb_old=old.txg, leaf_v=old.leaf_v)

    def op_crash(self, k):
        if k >= len(self.readable):
            return False
        lost = self.readable[-k:] if k else []
        self.readable = self.readable[:len(self.readable) - k] if k else self.readable
        sel_idx = None
        for idx in reversed(self.readable):
            r = self.roots[idx]
            if r.inst in self.rows and r.txg > self.rows[r.inst]:
                continue
            sel_idx = idx
            break
        if sel_idx is None:
            return False
        sel = self.roots[sel_idx]
        old_inst = self.inst
        self.inst += 1
        self.rows[sel.inst] = sel.txg
        # 恢复给 [max(所选根的实例, 1), 新实例) 每个实例写行、后写覆盖：严格介于两者之间的写 (i, 0, 0)（D18 已定项 11「恢复写的行」）
        for i in range(sel.inst + 1, self.inst):
            self.rows[i] = 0
        self.cur = sel_idx
        self.next_txg = max(self.roots[i].txg for i in self.ring()) + 1
        # 新实例的第一次发布（写行那次）：不改用户状态
        self.publish(dict(sel.objs), sel.wm, sel.tomb, sel.meta, {}, {}, False, leaf_v=sel.leaf_v)
        return True

    # ---- 扫描（按条款推）
    def newest_versions(self, versions, keyf):
        best = {}
        for v in versions:
            k = keyf(v)
            birth, inst, seq = v[1], v[2], v[3]
            if not self.published(birth, inst):
                continue
            if k not in best or (birth, inst, seq) > (best[k][1], best[k][2], best[k][3]):
                best[k] = v
        return best

    def scan_dead(self, arm):
        vers = self.disk[arm]
        if arm == "old_rw_unref":
            vers = [v for v in vers if not v[5]]   # 最坏情形：没有结构指着的空版本已被覆盖
        best = self.newest_versions(vers, lambda v: v[0])
        recs = [rec for v in best.values() for rec in v[4]]
        mbest = None
        for (birth, inst, seq, mrec) in self.meta_disk[arm]:
            if self.published(birth, inst) and (mbest is None or (birth, inst, seq) > mbest[0]):
                mbest = ((birth, inst, seq), mrec)
        meta = mbest[1] if mbest else ()
        dead = set()
        if arm == "cur_int":
            ints = [(a, b) for (t, a, b) in meta if t == "int"]
            for (n, b, d) in recs:
                if not any(a < d <= e for (a, e) in ints):
                    dead.add((n, b))
        elif arm == "cur_lin":
            parent = {t_rb: t_old for (t, t_rb, t_old) in meta if t == "lin"}
            lineage = set()
            t = self.roots[self.cur].txg
            while t >= 1:
                lineage.add(t)
                t = parent[t] if t in parent else t - 1
            for (n, b, d) in recs:
                if d in lineage:
                    dead.add((n, b))
        else:
            dead = set(recs)
        return dead

    def scan_leaf(self):
        best = None
        for i, v in enumerate(self.leaf_disk):
            if self.published(v[0], v[1]) and (best is None or (v[0], v[1], v[2]) > best[0]):
                best = ((v[0], v[1], v[2]), v[3])
        return best[1]

    def lineage(self):
        # 沿父链走出挂载根的谱系：谱系记录取 cur_lin 臂那一份（回退那次发布写的「T_rb 的父 = T_old」），崩溃恢复抛弃的那段由已发布谓词滤掉
        mbest = None
        for (birth, inst, seq, mrec) in self.meta_disk["cur_lin"]:
            if self.published(birth, inst) and (mbest is None or (birth, inst, seq) > mbest[0]):
                mbest = ((birth, inst, seq), mrec)
        parent = {t_rb: t_old for (t, t_rb, t_old) in (mbest[1] if mbest else ()) if t == "lin"}
        lin = set()
        t = self.roots[self.cur].txg
        while t >= 1:
            lin.add(t)
            t = parent[t] if t in parent else t - 1
        return lin

    def scan_leaf_on_lineage(self):
        lin = self.lineage()
        best = None
        for v in self.leaf_disk:
            if self.published(v[0], v[1]) and v[0] in lin and (best is None or (v[0], v[1], v[2]) > best[0]):
                best = ((v[0], v[1], v[2]), v[3])
        return best[1]

    def judge(self):
        truth = self.roots[self.cur].objs
        out = {}
        for arm in ARMS:
            dead = self.scan_dead(arm)
            dja = sum(1 for k, s in truth.items() if s == "dead" and k not in dead)
            ljd = sum(1 for k, s in truth.items() if s == "live" and k in dead)
            out[arm] = (dja, ljd)
        leaf = self.scan_leaf()
        ljd = sum(1 for k, s in truth.items() if s == "live" and leaf.get(k) != "live")
        dja = sum(1 for k, s in truth.items() if s == "dead" and leaf.get(k) == "live")
        out["inode_leaf"] = (dja, ljd)
        leaf = self.scan_leaf_on_lineage()
        ljd = sum(1 for k, s in truth.items() if s == "live" and leaf.get(k) != "live")
        dja = sum(1 for k, s in truth.items() if s == "dead" and leaf.get(k) == "live")
        out["inode_leaf_lin"] = (dja, ljd)
        return out


def moves(w):
    r = w.roots[w.cur]
    ms = ["mk", "nop"]
    ms += ["rm:%d:%d" % k for k, s in sorted(r.objs.items()) if s == "live"]
    ms += ["rb:%d" % w.roots[i].txg + "@%d" % i for i in w.candidates()]
    ms += ["crash:1", "crash:2", "crash:3"]
    return ms


def apply(w, m):
    if m == "mk":
        w.op_mk()
    elif m == "nop":
        w.op_nop()
    elif m.startswith("rm:"):
        _, n, b = m.split(":")
        w.op_rm((int(n), int(b)))
    elif m.startswith("rb:"):
        w.op_rb(int(m.split("@")[1]))
    elif m.startswith("crash:"):
        return w.op_crash(int(m.split(":")[1]))
    return True


def replay(seq):
    w = World()
    for m in seq:
        if not apply(w, m):
            return None
    return w


def main():
    import copy
    mode = sys.argv[1]
    if mode == "named":
        named = {
            "H1_delete_then_rollback_before_it": ["mk", "rm:1:2", "rb:2@1"],
            "H2_delete_rollback_delete_again": ["mk", "rm:1:2", "rb:2@1", "rm:1:2"],
            "H3_back_and_forth_rollback": ["mk", "rm:1:2", "nop", "rb:2@1", "rb:4@3"],
            "H4_container_rollover_then_rollback": ["mk", "mk", "mk", "rm:1:2", "rm:2:3", "rm:3:4", "rb:6@5"],
            "H5_crash_reissue": ["mk", "mk", "rm:2:3", "crash:2", "mk"],
            "H6_crash_reissue_same_number_and_birth": ["mk", "nop", "mk", "rm:2:4", "crash:3", "mk"],
        }
        for name, seq in named.items():
            w = replay(seq)
            res = w.judge()
            cells = " ".join("%s=DJA%d/LJD%d" % (a, *res[a]) for a in ARMS + ["inode_leaf", "inode_leaf_lin"])
            print("E name=named history=%s seq=%s %s" % (name, ",".join(seq), cells))
        return
    if mode == "exhaustive":
        depth = int(sys.argv[2])
        totals = defaultdict(lambda: [0, 0, 0])   # arm -> [histories with DJA, with LJD, histories]
        first = {}
        count = 0
        stack = [()]
        while stack:
            seq = stack.pop()
            w = replay(seq)
            if w is None:
                continue
            if seq:
                count += 1
                res = w.judge()
                for a, (dja, ljd) in res.items():
                    t = totals[a]
                    t[2] += 1
                    if dja:
                        t[0] += 1
                        first.setdefault((a, "DJA"), seq)
                    if ljd:
                        t[1] += 1
                        first.setdefault((a, "LJD"), seq)
            if len(seq) < depth:
                for m in moves(w):
                    stack.append(seq + (m,))
        print("E name=exhaustive depth=%d cap=%d ring=%d histories=%d" % (depth, CAP, RING, count))
        for a in ARMS + ["inode_leaf", "inode_leaf_lin"]:
            t = totals[a]
            print("E name=arm arm=%s histories_with_DJA=%d histories_with_LJD=%d histories=%d first_DJA=%s first_LJD=%s" % (
                a, t[0], t[1], t[2], ",".join(first.get((a, "DJA"), ("-",))), ",".join(first.get((a, "LJD"), ("-",)))))
        return
    if mode == "random":
        n, length, seed = int(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4])
        rng = random.Random(seed)
        totals = defaultdict(lambda: [0, 0, 0])
        first = {}
        for _ in range(n):
            w = World()
            seq = []
            for _ in range(length):
                m = rng.choice(moves(w))
                if not apply(w, m):
                    break
                seq.append(m)
                res = w.judge()
                for a, (dja, ljd) in res.items():
                    t = totals[a]
                    t[2] += 1
                    if dja:
                        t[0] += 1
                        first.setdefault((a, "DJA"), tuple(seq))
                    if ljd:
                        t[1] += 1
                        first.setdefault((a, "LJD"), tuple(seq))
        print("E name=random walks=%d length=%d seed=%d cap=%d ring=%d" % (n, length, seed, CAP, RING))
        for a in ARMS + ["inode_leaf", "inode_leaf_lin"]:
            t = totals[a]
            print("E name=arm_random arm=%s steps_with_DJA=%d steps_with_LJD=%d steps=%d first_DJA=%s first_LJD=%s" % (
                a, t[0], t[1], t[2], ",".join(first.get((a, "DJA"), ("-",))), ",".join(first.get((a, "LJD"), ("-",)))))


if __name__ == "__main__":
    main()
