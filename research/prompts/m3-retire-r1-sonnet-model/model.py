#!/usr/bin/env python3
"""m3-retire-r1 正推腿的小模型：单头 inode 叶容器 + 退役树 + 谱系记录 + 崩溃恢复的纯 Python 模拟。

不读 crates/，不依赖 SINGLEFS_REPO。回答三件事（报告 K2、K3、K5、K6 引用）：
  A  写法：退役集合 = 上一版的叶身份集 − 这一版的叶身份集，死亡 txg = 这次发布的 txg（集合差）；
  B  读侧：扫描重建读者五种（D 不看退役树 / C 退役记录不看谱系 / A 候选只过已发布谓词、记录按死亡 txg 滤谱系 / B = A 加
     候选单元的诞生 txg 也要在父链上 / E = B 但退役记录不先按已发布谓词滤版本、各版记录并起来）在带借、吸收、删空、回退、崩溃（txg 重发与跳号）的随机历史上的复活与误判死亡；
  C  checker 候选判据 a b c d 在合法历史上红几次，在五种植入的写者错误上红几次。
"""
import random
import sys
from collections import defaultdict

HEAD = 65537          # 单头：出生树 = inode 树的树 ID（模型取常量，不进结论）
CAP = 4               # 叶容器容量（只供测试旋钮的角色）
RCAP = 3              # 退役容器容量
FAULTS = ["none", "forget_drop", "forget_absorb", "retire_on_borrow", "death_txg_minus_one", "duplicate_record"]


def lineage_chain(txg, lineage):
    """与 walk.rs 的 lineage_chain_of_a_version 同一个算法：按 T_rb 从大到小，T_rb 大于此刻的跳过，否则 [T_rb, 此刻] 在链上、跳到 T_old。"""
    segments = []
    now = txg
    for trb, told in sorted(lineage, key=lambda r: -r[0]):
        if trb > now:
            continue
        segments.append((trb, now))
        now = told
    segments.append((0, now))
    return segments


def on_chain(segments, t):
    return any(lo <= t <= hi for lo, hi in segments)


class State:
    def __init__(self):
        self.txg = 0
        self.inst = 1
        self.leaves = []        # [(id, recs tuple)]；id = (tree, 2, number, birth)
        self.retire = []        # [[cno, cbirth, [(id, d), ...]], ...]
        self.lineage = []       # [(T_rb, T_old)]
        self.nextino = 1

    def copy(self):
        s = State()
        s.txg, s.inst, s.nextino = self.txg, self.inst, self.nextino
        s.leaves = list(self.leaves)
        s.retire = [[c, b, list(r)] for c, b, r in self.retire]
        s.lineage = list(self.lineage)
        return s

    def ids(self):
        return {i for i, _ in self.leaves}


class World:
    def __init__(self, seed, policy, theta, fault="none", p_rollback=0.07, p_crash=0.05, alpha=False):
        self.alpha = alpha            # 备选写法：回退那次发布给“现行有、R_old 没有”的叶身份写退役记录（死亡 txg = T_rb）
        self.rng = random.Random(seed)
        self.p_rollback = p_rollback
        self.p_crash = p_crash
        self.policy = policy          # "drop" | "absorb" | "borrow"
        self.theta = theta
        self.fault = fault
        self.cur = State()
        self.published = []           # 所有发布成功的版本（含回退缺口里的）
        self.by_txg = {}
        self.disk_leaf = []           # (id, ubirth, inst, txn, recs)
        self.disk_ret = []            # (cno, ubirth, inst, txn, records tuple)
        self.table = {}               # 实例 -> T_pub（崩溃恢复写的行）
        self.txn = 0
        self.scans = []               # 每次扫描的 (名字, 版本, 磁盘快照长度, 表快照)
        self.counts = defaultdict(int)

    # ---- 写 ----
    def write_leaf(self, leaf_id, recs, st):
        self.txn += 1
        self.disk_leaf.append((leaf_id, st.txg, st.inst, self.txn, tuple(recs)))

    def append_retire(self, st, records):
        """把记录追加进开放退役容器（满了才开新容器，号 = 最大 + 1，出生代 = 本次 txg）；返回被重写的容器下标。"""
        touched = []
        for rec in records:
            if st.retire and len(st.retire[-1][2]) < RCAP:
                st.retire[-1][2].append(rec)
            else:
                cno = st.retire[-1][0] + 1 if st.retire else 1
                st.retire.append([cno, st.txg, [rec]])
            if len(st.retire) - 1 not in touched:
                touched.append(len(st.retire) - 1)
        for idx in touched:
            cno, cb, recs = st.retire[idx]
            self.txn += 1
            self.disk_ret.append((cno, st.txg, st.inst, self.txn, tuple(recs)))
        return touched

    def publish(self, st, before_ids, rewritten, retire_from_diff=True, extra_retire=None, abandon=False):
        """st.txg 已加一。rewritten = 这次重写的叶下标；退役集合 = 集合差（可被植入的写者错误改写）。"""
        for idx in rewritten:
            self.write_leaf(st.leaves[idx][0], st.leaves[idx][1], st)
        diff = sorted(before_ids - st.ids())
        records = [(i, st.txg) for i in diff]
        if extra_retire:
            records += extra_retire
        self.append_retire(st, records)
        if not abandon:
            self.cur = st
            self.published.append(st.copy())
            self.by_txg[st.txg] = self.published[-1]
        return diff

    # ---- 用户操作 ----
    def op_create(self, st, count):
        st = st.copy()
        st.txg += 1
        before = st.ids()
        rewritten = set()
        for _ in range(count):
            ino = st.nextino
            st.nextino += 1
            if st.leaves and len(st.leaves[-1][1]) < CAP:
                lid, recs = st.leaves[-1]
                st.leaves[-1] = (lid, recs + (ino,))
                rewritten.add(len(st.leaves) - 1)
            else:
                st.leaves.append(((HEAD, 2, ino, st.txg), (ino,)))
                rewritten.add(len(st.leaves) - 1)
        return st, before, sorted(rewritten), []

    def op_delete(self, st, ino):
        st = st.copy()
        st.txg += 1
        before = st.ids()
        extra = []
        c = next(i for i, (_, recs) in enumerate(st.leaves) if ino in recs)
        lid, recs = st.leaves[c]
        recs = tuple(r for r in recs if r != ino)
        st.leaves[c] = (lid, recs)
        rewritten = {c}
        action = "none"
        if not recs:
            action = "drop"
            del st.leaves[c]
            rewritten = set(range(c, len(st.leaves)))
        elif self.policy != "drop" and len(recs) < self.theta:
            right = st.leaves[c + 1] if c + 1 < len(st.leaves) else None
            left = st.leaves[c - 1] if c > 0 else None
            if right is not None and len(recs) + len(right[1]) <= CAP:
                action = "absorb"
                st.leaves[c] = (lid, recs + right[1])
                del st.leaves[c + 1]
                rewritten = set(range(c, len(st.leaves)))
            elif right is not None and self.policy == "borrow" and (len(right[1]) - len(recs)) // 2 >= 1:
                k = (len(right[1]) - len(recs)) // 2
                action = "borrow"
                st.leaves[c] = (lid, recs + right[1][:k])
                st.leaves[c + 1] = (right[0], right[1][k:])     # 右片保留身份
                rewritten = {c, c + 1}
                if self.fault == "retire_on_borrow":
                    extra.append((right[0], st.txg))
            elif right is None and left is not None and len(left[1]) + len(recs) <= CAP:
                action = "absorb"
                st.leaves[c - 1] = (left[0], left[1] + recs)
                del st.leaves[c]
                rewritten = {c - 1}
        self.counts["action_" + action] += 1
        # 植入的写者错误：改写“集合差”这一步
        if (self.fault == "forget_drop" and action == "drop") or (self.fault == "forget_absorb" and action == "absorb"):
            self._skip_diff = True
        else:
            self._skip_diff = False
        return st, before, sorted(rewritten), extra

    def finish(self, st, before, rewritten, extra):
        """发布并按故障改写退役记录。"""
        diff = sorted(before - st.ids())
        self.counts["publishes_with_retirement"] += 1 if diff else 0
        self.counts["max_retire_records_in_one_publish"] = max(self.counts["max_retire_records_in_one_publish"], len(diff))
        records = [] if self._skip_diff_now() else [(i, st.txg) for i in diff]
        if self.fault == "death_txg_minus_one":
            records = [(i, d - 1) for i, d in records]
        if self.fault == "duplicate_record":
            records = records + records
        for idx in rewritten:
            self.write_leaf(st.leaves[idx][0], st.leaves[idx][1], st)
        touched = self.append_retire(st, records + extra)
        self.counts["max_retire_containers_rewritten_in_one_publish"] = max(self.counts["max_retire_containers_rewritten_in_one_publish"], len(touched))
        self.counts["retire_records_total"] += len(records + extra)
        self.cur = st
        self.published.append(st.copy())
        self.by_txg[st.txg] = self.published[-1]
        self._skip_diff = False

    def _skip_diff_now(self):
        return getattr(self, "_skip_diff", False)

    def do_rollback(self):
        chain = lineage_chain(self.cur.txg, self.cur.lineage)
        cands = [v for v in self.published if on_chain(chain, v.txg) and v.txg < self.cur.txg and v.leaves]
        if not cands:
            return False
        target = self.rng.choice(cands[-12:])
        st = self.cur.copy()
        st.txg += 1
        st.leaves = list(target.leaves)
        st.lineage = st.lineage + [(st.txg, target.txg)]
        cur_only = len(self.cur.ids() - target.ids())
        self.counts["rollback_cur_only_ids_sum"] += cur_only
        self.counts["rollback_cur_only_ids_max"] = max(self.counts["rollback_cur_only_ids_max"], cur_only)
        if self.alpha:
            self.append_retire(st, [(i, st.txg) for i in sorted(self.cur.ids() - target.ids())])
        self.cur = st
        self.published.append(st.copy())
        self.by_txg[st.txg] = self.published[-1]
        self.counts["rollback"] += 1
        return True

    def do_crash(self, abandoned_ops):
        """崩溃：在老实例上发 abandoned_ops 次发布，单元落盘、根没写成；恢复 = 新实例，行 (老实例, 所选根 txg)，txg 重发或跳号。"""
        shadow = self.cur.copy()
        for _ in range(abandoned_ops):
            st, before, rewritten, extra = self.random_user_change(shadow)
            if st is None:
                break
            diff = sorted(before - st.ids())
            for idx in rewritten:
                self.write_leaf(st.leaves[idx][0], st.leaves[idx][1], st)
            self.append_retire(st, [(i, st.txg) for i in diff] + extra)
            shadow = st
        selected = self.cur
        self.table[selected.inst] = selected.txg
        self.counts["crash"] += 1
        self.scan("recovery", selected, new_inst=selected.inst)
        new_state = selected.copy()
        new_state.inst = selected.inst + 1
        jump = self.rng.choice([0, 0, abandoned_ops])
        new_state.txg = selected.txg + jump
        self.cur = new_state
        self.counts["crash_jump_%d" % (1 if jump else 0)] += 1

    def random_user_change(self, st):
        r = self.rng.random()
        if r < 0.45 or not st.leaves:
            return self.op_create(st, self.rng.randint(1, 5))
        inos = [x for _, recs in st.leaves for x in recs]
        return self.op_delete(st, self.rng.choice(inos))

    # ---- 读侧（扫描重建读者）----
    def published_predicate(self, inst, birth, root):
        if inst > root.inst:
            return False
        if inst == root.inst:
            return birth <= root.txg
        return inst not in self.table or birth <= self.table[inst]

    def reader(self, root, variant):
        """返回 (活身份集, {id: 选中的记录})。variant: D 不看退役 / C 记录不看谱系 / A / B。"""
        chain = lineage_chain(root.txg, root.lineage)
        candidates = defaultdict(list)
        for lid, ub, inst, txn, recs in self.disk_leaf:
            if not self.published_predicate(inst, ub, root):
                continue
            if variant in ("B", "E") and not on_chain(chain, ub):
                continue
            candidates[lid].append((ub, inst, txn, recs))
        selected = {lid: max(vs) for lid, vs in candidates.items()}
        retired = []
        if variant == "E":
            # 读者 E = 读者 B 的候选与择新，但退役记录不先按已发布谓词滤版本、把各版（含被抛弃段里写的）的记录并起来再按死亡 txg 滤谱系
            for cno, ub, inst, txn, recs in self.disk_ret:
                retired.extend(recs)
        elif variant != "D":
            groups = defaultdict(list)
            for cno, ub, inst, txn, recs in self.disk_ret:
                if self.published_predicate(inst, ub, root):
                    groups[cno].append((ub, inst, txn, recs))
            for vs in groups.values():
                newest = max(vs)
                retired.extend(newest[3])
        killed = set()
        for lid, d in retired:
            if variant != "C" and not on_chain(chain, d):
                continue
            if lid in selected and selected[lid][0] <= d:
                killed.add(lid)
        live = {lid: v[3] for lid, v in selected.items() if lid not in killed}
        return live

    def scan(self, name, root, new_inst=None):
        truth = {lid: recs for lid, recs in root.leaves}
        for variant in ["D", "C", "A", "B", "E"]:
            live = self.reader(root, variant)
            resurrect = len(set(live) - set(truth))
            misdeath = len(set(truth) - set(live))
            wrong = sum(1 for lid in set(live) & set(truth) if live[lid] != truth[lid])
            self.counts["scan_" + variant] += 1
            self.counts["resurrect_" + variant] += resurrect
            self.counts["misdeath_" + variant] += misdeath
            self.counts["wrong_content_" + variant] += wrong
            if resurrect or misdeath or wrong:
                self.counts["scans_with_error_" + variant] += 1
        if not self.alpha:
            self.check_invariants(root)

    # ---- checker 候选判据 ----
    def check_invariants(self, v):
        chain = lineage_chain(v.txg, v.lineage)
        ids_v = v.ids()
        records = [(i, d) for _, _, rs in v.retire for (i, d) in rs]
        red = set()
        for i, d in records:
            if not (i[1] == 2 and i[0] != 0 and i[3] < d <= v.txg):
                red.add("a")
        on_lineage = [(i, d) for i, d in records if on_chain(chain, d)]
        seen = set()
        for i, d in on_lineage:
            if i in seen:
                red.add("b")
            seen.add(i)
        for i, d in on_lineage:
            if i in ids_v:
                red.add("c")
        if any(t == v.txg for t, _ in v.lineage):
            parent_txg = next(o for t, o in v.lineage if t == v.txg)
        else:
            parent_txg = v.txg - 1
        parent = self.by_txg.get(parent_txg)
        if parent is not None and parent.leaves is not None and parent_txg > 0:
            gone = parent.ids() - ids_v
            died_now = {i for i, d in on_lineage if d == v.txg}
            if gone - died_now:
                red.add("d_missing")
            if died_now - gone:
                red.add("d_extra")
        self.counts["inv_checked"] += 1
        for r in sorted(red):
            self.counts["inv_red_" + r] += 1
        if red:
            self.counts["inv_versions_red"] += 1


def run_history(seed, policy, theta, fault, ops, p_rollback=0.07, p_crash=0.05, alpha=False):
    w = World(seed, policy, theta, fault, p_rollback, p_crash, alpha)
    w.cur = State()
    w.cur.txg = 2
    w.cur.inst = 1
    st, before, rewritten, extra = w.op_create(w.cur, 3)
    w.finish(st, before, rewritten, extra)
    w.scan("first", w.cur)
    for _ in range(ops):
        r = w.rng.random()
        if r < w.p_rollback:
            if w.do_rollback():
                w.scan("rollback", w.cur)
            continue
        if r < w.p_rollback + w.p_crash:
            w.do_crash(w.rng.randint(1, 3))
            continue
        st, before, rewritten, extra = w.random_user_change(w.cur)
        if st is None:
            continue
        w.finish(st, before, rewritten, extra)
        w.scan("publish", w.cur)
    return w.counts


def aggregate(histories, policy, theta, fault, ops, p_rollback, p_crash, alpha=False):
    total = defaultdict(int)
    for seed in range(histories):
        for k, v in run_history(seed, policy, theta, fault, ops, p_rollback, p_crash, alpha).items():
            if k.startswith("max_") or k.endswith("_max"):
                total[k] = max(total[k], v)
            else:
                total[k] += v
    return total


def main():
    histories = int(sys.argv[1]) if len(sys.argv) > 1 else 200
    ops = int(sys.argv[2]) if len(sys.argv) > 2 else 60
    print("MODEL histories=%d ops=%d CAP=%d RCAP=%d" % (histories, ops, CAP, RCAP))
    # 一：合法历史（无植入），三种合并政策 × 四种历史形（有无回退、有无崩溃）
    for policy, theta in [("drop", 0), ("absorb", 2), ("borrow", 2)]:
        for shape, p_rb, p_cr in [("full", 0.07, 0.05), ("no_rollback", 0.0, 0.05), ("no_crash", 0.07, 0.0), ("neither", 0.0, 0.0)]:
            total = aggregate(histories, policy, theta, "none", ops, p_rb, p_cr)
            print("RESULT group=legal policy=%s theta=%d shape=%s %s" % (policy, theta, shape, " ".join("%s=%d" % (k, total[k]) for k in sorted(total))))
    # 一之二：备选 α（回退那次发布给现行独有的叶身份写退役记录），读者 A 与 B 的复活、误判死亡、内容错
    total = aggregate(histories, "borrow", 2, "none", ops, 0.07, 0.05, True)
    print("RESULT group=alpha_rollback_retires policy=borrow theta=2 shape=full %s" % " ".join("%s=%d" % (k, total[k]) for k in sorted(total)))
    # 二：五种植入的写者错误（borrow 政策、有回退有崩溃）
    for fault in FAULTS[1:]:
        total = aggregate(histories, "borrow", 2, fault, ops, 0.07, 0.05)
        print("RESULT group=fault policy=borrow theta=2 fault=%s %s" % (fault, " ".join("%s=%d" % (k, total[k]) for k in sorted(total))))


if __name__ == "__main__":
    main()
