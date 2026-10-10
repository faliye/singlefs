#!/usr/bin/env python3
"""m3-retire-r1 攻方腿（opus）的抽象模型：单头、N1、均分借、左吸收右，退役树照墓碑树（容器号从 1 起、有新记录才重写、
满了才关、不回收、根满长高），管理员回退（退役树、谱系树照抄 cur；inode 树取 R_old；水位取 max），崩溃恢复（抛弃末尾 m 次发布，
写实例表行，新 txg = 盘上见过的最大 + 1，水位随所选根）。不依赖 crates/、不读盘字节；粒度是一次发布。

判三样：
1. 扫描重建（X1 甲）：叶身份有一版已发布（P2）且诞生代号在这一版谱系上（I-9.18 的父链：谱系记录跳 T_old、其余 txg − 1）
   即候选；退役容器按 P2 + 择新（P3）读，退役记录按死亡 txg 在谱系上滤。对照臂：不带死亡 txg（A1）、带 txg 不滤谱系（A2）。
2. checker 候选判据（K5）：C1 退役身份不在现行叶（字面，不滤）、C1L（按谱系滤）、C2 死亡 txg ≤ 这一版 txg、C3 死亡 txg ≥ 容器出生代、
   C3S 严格大于、C4 退役身份不重复（字面）、C4L（只判谱系上的）、C6 非最右退役容器装满、C7 退役容器号 1..n 连号。
3. 准入（K4）：每次发布退役记录条数、退役树写的块数对「容器 2 + 根 1 + 内部层数」（层数取发布之前 / 之后），
   永久占用对两种预留（P-a 只进 ckpt_cost 的暂态项、不预付；P-b 墓碑式：常驻 + 每活对象一份）的缺口。
"""
import sys
import random

SLOT = 16384
RECORD_BYTES = 34  # 26 身份引用 + 8 死亡 txg


class Config:
    def __init__(self, leaf_cap, theta, retire_cap, fanout, ring, death_at_intent=False):
        self.death_at_intent = death_at_intent  # 对照写法：死亡 txg 取 unlink 意图那次（不是摘掉 / 吸收那次）
        self.leaf_cap = leaf_cap
        self.theta = theta
        self.retire_cap = retire_cap
        self.fanout = fanout
        self.ring = ring

    def name(self):
        death = "-death_at_intent" if self.death_at_intent else ""
        return f"cap{self.leaf_cap}-th{self.theta}-rc{self.retire_cap}-f{self.fanout}-ring{self.ring}{death}"


_SHAPES = {}


def shape_after(n_containers, fanout):
    """照 crates/singlefs-core/src/container_tree.rs 的 after_a_container_is_opened_at_the_right_end 逐片往右挂（扇出固定时形状只是片数的函数）。"""
    key = (n_containers, fanout)
    if key in _SHAPES:
        return _SHAPES[key]
    if n_containers == 0:
        levels = []
    else:
        prev = [list(level) for level in shape_after(n_containers - 1, fanout)]
        levels = prev
        i = 0
        while True:
            if i >= len(levels):
                levels.append([1])
                break
            top = i + 1 == len(levels)
            if levels[i][-1] < fanout:
                levels[i][-1] += 1
                break
            levels[i].append(1)
            if top:
                levels.append([2])
                break
            i += 1
    result = tuple(tuple(level) for level in levels)
    _SHAPES[key] = result
    return result


def internal_levels(n_containers, fanout):
    return max(len(shape_after(n_containers, fanout)) - 1, 0)


def node_count(n_containers, fanout):
    return sum(len(level) for level in shape_after(n_containers, fanout))


def nodes_rewritten_on_append(n_before, n_after, fanout):
    """一次追加之后重写的码 2 节点：最右那一条路径（根满长高那次新根也在路径上）。"""
    if n_after == 0:
        return 0
    return len(shape_after(n_after, fanout))


class Version:
    __slots__ = ("txg", "inst", "parent", "leaves", "queue", "retire", "lineage", "wm", "rows", "judged", "intent")

    def __init__(self, txg, inst, parent, leaves, queue, retire, lineage, wm, rows, judged, intent=0):
        self.intent = intent      # 队列里那个号的意图发布的 txg（队列空时 0）
        self.txg = txg
        self.inst = inst
        self.parent = parent      # 这一版是接在哪一版后面发的（回退那次是 cur，不是 R_old）
        self.leaves = leaves      # ((容器号, 出生代, (号, …), (nlink=0 的号, …)), …) 按叶序
        self.queue = queue        # deleted_inodes：至多一个号
        self.retire = retire      # ((容器号, 出生代, (((叶容器号, 叶出生代), 死亡 txg), …)), …)
        self.lineage = lineage    # ((T_rb, T_old), …)
        self.wm = wm              # inode 号水位（下一个号）
        self.rows = rows          # 实例表：((实例, T_pub), …)
        self.judged = judged      # 这一版是不是准入式子判过的写入类发布


def leaf_ident(leaf):
    return (leaf[0], leaf[1])


def retire_records(version):
    return [record for container in version.retire for record in container[2]]


def retire_footprint_slots(n_records, cfg):
    containers = -(-n_records // cfg.retire_cap)
    return 2 * containers + node_count(containers, cfg.fanout)


def lineage_set(version):
    """I-9.18 的「在谱系上」：从这一版的 txg 起，有谱系记录 (T_rb, T_old) 的 T_rb 父是 T_old，其余 txg 的父是 txg − 1。"""
    jumps = dict(version.lineage)
    seen = set()
    t = version.txg
    while t > 0 and t not in seen:
        seen.add(t)
        t = jumps.get(t, t - 1)
    return seen


def n_records(version):
    return sum(len(leaf[2]) for leaf in version.leaves)


class Pool:
    __slots__ = ("cfg", "versions", "ring", "cur", "next_txg", "max_inst", "leaf_units", "retire_units",
                 "free", "log")

    def copy(self):
        other = Pool.__new__(Pool)
        other.cfg = self.cfg
        other.versions = dict(self.versions)
        other.ring = dict(self.ring)
        other.cur = self.cur
        other.next_txg = self.next_txg
        other.max_inst = self.max_inst
        other.leaf_units = dict(self.leaf_units)
        other.retire_units = dict(self.retire_units)
        other.free = self.free
        other.log = self.log
        return other

    @staticmethod
    def initial(cfg, creates):
        pool = Pool.__new__(Pool)
        pool.cfg = cfg
        pool.versions = {}
        pool.ring = {}
        pool.cur = Version(2, 1, None, (), (), (), (), 1, (), True)
        pool.next_txg = 3
        pool.max_inst = 1
        pool.leaf_units = {}
        pool.retire_units = {}
        pool.free = {}
        pool.log = ()
        for _ in range(creates):
            pool = pool.create()
        return pool

    # ---- 写路径 ----
    def _publish(self, leaves, queue, retired, rewritten_leaves, judged, kind, retire=None, lineage=None, wm=None,
                 rows=None, inst=None, parent=None, txg=None, base_version=None, intent=0, death=None):
        cfg = self.cfg
        pool = self.copy()
        cur = self.cur
        T = self.next_txg if txg is None else txg
        inst = cur.inst if inst is None else inst
        containers = list(cur.retire if retire is None else retire)
        rewritten_retire = []
        n_before = len(containers)
        death = T if death is None else death
        for ident in retired:
            if containers and len(containers[-1][2]) < cfg.retire_cap:
                last = containers[-1]
                containers[-1] = (last[0], last[1], last[2] + ((ident, death),))
            else:
                number = containers[-1][0] + 1 if containers else 1
                containers.append((number, T, ((ident, death),)))
            if len(containers) - 1 not in rewritten_retire:
                rewritten_retire.append(len(containers) - 1)
        retire_t = tuple(containers)
        version = Version(T, inst, cur if parent is None else parent, leaves, queue, retire_t,
                          cur.lineage if lineage is None else lineage, cur.wm if wm is None else wm,
                          cur.rows if rows is None else rows, judged, intent)
        for leaf in leaves:
            if leaf_ident(leaf) in rewritten_leaves:
                key = leaf_ident(leaf)
                pool.leaf_units[key] = pool.leaf_units.get(key, ()) + ((T, inst),)
        for index in rewritten_retire:
            c = containers[index]
            key = (c[0], c[1])
            pool.retire_units[key] = pool.retire_units.get(key, ()) + ((T, inst, c[2]),)
        pool.versions[T] = version
        pool.ring[T % cfg.ring] = T
        pool.cur = version
        pool.next_txg = T + 1
        pool.max_inst = max(pool.max_inst, inst)
        # 准入：退役树这一次写的块数，对「容器 2 + 根 1 + 内部层数」
        n_after = len(containers)
        written = 0
        if rewritten_retire:
            written = 2 * len(rewritten_retire) + nodes_rewritten_on_append(n_before, n_after, cfg.fanout)
        h_before = internal_levels(n_before, cfg.fanout)
        h_after = internal_levels(n_after, cfg.fanout)
        # 可用（只记退役树这一维）：准入判过的写入类发布之后取最坏——正好贴着这一版的预留（P-a 0、P-b reserve_b）；
        # 不判式子的发布接在哪一版的可用上：删除类接 cur，回退接 R_old 那一版当时的可用（R_old 之后别的空间随回退回来），
        # 崩溃恢复接所选根；再减退役树比那一版多占的槽（照抄 cur 带过来的那一段也算）。
        if judged:
            free_pair = (0, reserve_b(version, cfg))
        else:
            base = cur if base_version is None else base_version
            fa, fb = self.free.get(base.txg, (0, 0))
            growth = retire_footprint_slots(len(retire_records(version)), cfg) - retire_footprint_slots(
                len(retire_records(base)), cfg)
            free_pair = (fa - growth * SLOT, fb - growth * SLOT)
        pool.free = dict(self.free)
        pool.free[T] = free_pair
        pool.log = self.log + ((kind, T, len(retired), written, 3 + h_before, 3 + h_after),)
        return pool

    def create(self):
        cur = self.cur
        assert not cur.queue
        n = cur.wm
        T = self.next_txg
        leaves = list(cur.leaves)
        if leaves and len(leaves[-1][2]) < self.cfg.leaf_cap:
            last = leaves[-1]
            leaves[-1] = (last[0], last[1], last[2] + (n,))
        else:
            leaves.append((n, T, (n,)))
        return self._publish(tuple(leaves), (), [], {leaf_ident(leaves[-1])}, True, "create", wm=n + 1)

    def intent(self, x):
        cur = self.cur
        assert not cur.queue
        holder = next(leaf for leaf in cur.leaves if x in leaf[2])
        return self._publish(cur.leaves, (x,), [], {leaf_ident(holder)}, False, f"intent{x}", intent=self.next_txg)

    def final(self):
        """unlink 最后一批：删掉那一条，照 M(θ) 合并（E178 装置的写法）：删空不落盘；|C| < θ 时有右邻就吸收或均分借，
        C 是最右一片就让左邻吸收。身份消失（删空、被吸收）那次给它写一条退役记录；借（N1）右片保留身份、不写。"""
        cfg = self.cfg
        cur = self.cur
        (x,) = cur.queue
        leaves = [list(leaf) for leaf in cur.leaves]
        i = next(k for k, leaf in enumerate(leaves) if x in leaf[2])
        leaves[i][2] = tuple(r for r in leaves[i][2] if r != x)
        retired = []
        rewritten = {(leaves[i][0], leaves[i][1])}
        if not leaves[i][2]:
            retired.append((leaves[i][0], leaves[i][1]))
            del leaves[i]
        elif len(leaves[i][2]) < cfg.theta:
            if i + 1 < len(leaves):
                right = leaves[i + 1]
                if len(leaves[i][2]) + len(right[2]) <= cfg.leaf_cap:
                    leaves[i][2] = leaves[i][2] + right[2]
                    retired.append((right[0], right[1]))
                    del leaves[i + 1]
                else:
                    k = -(-(len(right[2]) - len(leaves[i][2])) // 2)
                    leaves[i][2] = leaves[i][2] + right[2][:k]
                    right[2] = right[2][k:]
                    rewritten.add((right[0], right[1]))
            elif i > 0:
                left = leaves[i - 1]
                if len(left[2]) + len(leaves[i][2]) <= cfg.leaf_cap:
                    left[2] = left[2] + leaves[i][2]
                    retired.append((leaves[i][0], leaves[i][1]))
                    rewritten.add((left[0], left[1]))
                    del leaves[i]
        death = cur.intent if cfg.death_at_intent else None
        return self._publish(tuple(tuple(leaf) for leaf in leaves), (), retired, rewritten, False, f"final{x}",
                             death=death)

    def valid_by_table(self, version, rows):
        table = dict(rows)
        return version.inst not in table or version.txg <= table[version.inst]

    def ring_roots(self):
        return [self.versions[t] for t in sorted(self.ring.values())]

    def rollback_candidates(self):
        cur = self.cur
        return [v for v in self.ring_roots() if v.txg < cur.txg and self.valid_by_table(v, cur.rows)]

    def rollback(self, target):
        """管理员回退：一次向前发布；inode 树与 deleted_inodes 取 R_old，退役树、谱系树照抄 cur，追加 (T_rb, T_old)；
        水位取 max(cur, 环里没被抛弃的根)；不写实例表行；不判准入式子。"""
        cur = self.cur
        T = self.next_txg
        wm = max([cur.wm] + [v.wm for v in self.ring_roots() if self.valid_by_table(v, cur.rows)])
        return self._publish(target.leaves, target.queue, [], set(), False, f"rollback{target.txg}",
                             lineage=cur.lineage + ((T, target.txg),), wm=wm, base_version=target,
                             intent=target.intent)

    def crash(self, m):
        """崩溃恢复：cur 那次发布的根没落盘（m = 1），或连它前一次也被抛弃（m = 2，读错退回）。所选根要在环里、按 cur 的表有效。
        恢复写实例表行、发写行那一次（新实例、txg = 盘上见过的最大 + 1），树与水位随所选根。被抛弃那几次写的单元留在盘上。"""
        chain = [self.cur]
        for _ in range(m):
            if chain[-1].parent is None:
                return None
            chain.append(chain[-1].parent)
        chosen = chain[-1]
        if self.ring.get(chosen.txg % self.cfg.ring) != chosen.txg or not self.valid_by_table(chosen, self.cur.rows):
            return None
        if chosen.txg < 3:
            return None
        pool = self.copy()
        if pool.ring.get(self.cur.txg % self.cfg.ring) == self.cur.txg:
            del pool.ring[self.cur.txg % self.cfg.ring]
        new_inst = self.max_inst + 1
        rows = dict(chosen.rows)
        for i in range(max(chosen.inst, 1), new_inst):
            rows[i] = chosen.txg if i == chosen.inst else 0
        pool.cur = chosen
        return pool._publish(chosen.leaves, chosen.queue, [], set(), False, f"crash{m}", retire=chosen.retire,
                             lineage=chosen.lineage, wm=chosen.wm, rows=tuple(sorted(rows.items())), inst=new_inst,
                             parent=chosen, txg=self.next_txg, intent=chosen.intent)


def reserve_b(version, cfg):
    """P-b（墓碑式「解开自己」）：常驻 (2 + 1 + H + 1) 槽 + 每个记录（活的与队列里的）⌈(2 + 2 ÷ (扇出 − 1)) × 16384 ÷ 一片条数⌉ 字节。
    比墓碑那一份（2 + H + 1 槽、⌈2 × 16384 ÷ 815⌉）放得宽：要它在不回退的历史上不亏，亏只算回退带出来的那一份。"""
    containers = -(-len(retire_records(version)) // cfg.retire_cap)
    h = internal_levels(containers, cfg.fanout)
    per_object = -(-((2 * (cfg.fanout - 1) + 2) * SLOT) // ((cfg.fanout - 1) * cfg.retire_cap))
    return (2 + 1 + h + 1) * SLOT + per_object * n_records(version)


def true_need_slots(version, cfg):
    """从这一版起把树里每条记录都删掉、每次删都退役一个身份时，退役树最多再长多少槽（只增不减）。"""
    r = len(retire_records(version))
    return retire_footprint_slots(r + n_records(version), cfg) - retire_footprint_slots(r, cfg)


ARMS = ("A0", "A1", "A2")
RULES = ("C1", "C1L", "C2", "C3", "C3S", "C4", "C4L", "C6", "C7")


def judge(pool):
    """对 pool.cur 这一版判扫描重建三条臂与九条 checker 候选，外加准入两种预留的缺口。交回一个 dict。"""
    cfg = pool.cfg
    version = pool.cur
    lineage = lineage_set(version)
    rows = dict(version.rows)

    def published(t, inst):
        if inst == version.inst:
            return t <= version.txg
        if inst > version.inst:
            return False
        if inst in rows:
            return t <= rows[inst]
        return True

    scan = []
    for unit_versions in pool.retire_units.values():
        live = [u for u in unit_versions if published(u[0], u[1])]
        if live:
            scan.extend(max(live, key=lambda u: (u[0], u[1]))[2])
    truth = {leaf_ident(leaf) for leaf in version.leaves}
    candidates = {ident for ident, units in pool.leaf_units.items()
                  if any(published(t, i) and t in lineage for t, i in units)}
    kills = {
        "A0": {ident for ident, death in scan if death in lineage},
        "A1": {ident for ident, death in scan},
        "A2": {ident for ident, death in scan if death <= version.txg},
    }
    out = {}
    for arm in ARMS:
        rebuilt = candidates - kills[arm]
        out[arm + ".res"] = len(rebuilt - truth)
        out[arm + ".fd"] = len(truth - rebuilt)
    records = retire_records(version)
    on = [r for r in records if r[1] in lineage]
    idents = [r[0] for r in records]
    on_idents = [r[0] for r in on]
    numbers = [c[0] for c in version.retire]
    out["C1"] = int(any(r[0] in truth for r in records))
    out["C1L"] = int(any(r[0] in truth for r in on))
    out["C2"] = int(any(r[1] > version.txg for r in records))
    out["C3"] = int(any(r[1] < r[0][1] for r in records))
    out["C3S"] = int(any(r[1] <= r[0][1] for r in records))
    out["C4"] = int(len(idents) != len(set(idents)))
    out["C4L"] = int(len(on_idents) != len(set(on_idents)))
    out["C6"] = int(any(len(c[2]) != cfg.retire_cap for c in version.retire[:-1]))
    out["C7"] = int(numbers != list(range(1, len(numbers) + 1)))
    need = true_need_slots(version, cfg) * SLOT
    free_a, free_b = pool.free[version.txg]
    out["Pa.deficit"] = need - free_a
    out["Pb.deficit"] = need - free_b
    out["Pa.free"] = free_a
    out["Pb.free"] = free_b
    out["retire_records"] = len(records)
    out["same_ident_max"] = max([idents.count(i) for i in set(idents)] or [0])
    # 模型自查：叶非空、号沿叶序递增、记录号 ≥ 容器号
    keys = [r for leaf in version.leaves for r in leaf[2]]
    assert all(leaf[2] for leaf in version.leaves), "类型 2 容器不空"
    assert keys == sorted(keys) and len(keys) == len(set(keys)), "号沿叶序严格递增"
    assert all(min(leaf[2]) >= leaf[0] for leaf in version.leaves), "记录号 ≥ 容器号"
    return out


def operations(pool):
    cur = pool.cur
    ops = []
    if not cur.queue:
        ops.append(("create",))
        ops.extend(("intent", x) for leaf in cur.leaves for x in leaf[2])
    else:
        ops.append(("final",))
    ops.extend(("rb", v.txg) for v in pool.rollback_candidates())
    ops.extend(("crash", m) for m in (1, 2))
    return ops


def apply(pool, op):
    if op[0] == "create":
        return pool.create()
    if op[0] == "intent":
        return pool.intent(op[1])
    if op[0] == "final":
        return pool.final()
    if op[0] == "rb":
        return pool.rollback(pool.versions[op[1]])
    return pool.crash(op[1])


class Tally:
    def __init__(self):
        self.versions = 0
        self.counts = {}
        self.peaks = {}
        self.first = {}
        self.max_retired_per_publish = 0
        self.growth_short = 0
        self.max_written_minus_term_before = 0

    def add(self, pool, history):
        self.versions += 1
        result = judge(pool)
        for key, value in result.items():
            if key in ("Pa.free", "Pb.free", "retire_records"):
                self.peaks[key] = min(self.peaks.get(key, 0), value) if key != "retire_records" else max(
                    self.peaks.get(key, 0), value)
                continue
            if value > 0:
                self.counts[key] = self.counts.get(key, 0) + 1
                if key not in self.first:
                    self.first[key] = " ".join(":".join(str(p) for p in op) for op in history)
            self.peaks[key] = max(self.peaks.get(key, value), value)
        kind, txg, retired, written, term_before, term_after = pool.log[-1]
        self.max_retired_per_publish = max(self.max_retired_per_publish, retired)
        if written > term_before:
            self.growth_short += 1
        self.max_written_minus_term_before = max(self.max_written_minus_term_before, written - term_before)

    def lines(self, label):
        out = [f"{label} versions={self.versions} max_retired_per_publish={self.max_retired_per_publish} "
               f"growth_publishes_written_more_than_term_before={self.growth_short} "
               f"max_written_minus_term_before={self.max_written_minus_term_before}"]
        for key in [f"{a}.{s}" for a in ARMS for s in ("res", "fd")] + list(RULES) + ["same_ident_max"]:
            out.append(f"{label} {key} versions_positive={self.counts.get(key, 0)} peak={self.peaks.get(key, 0)}")
        for key in ("Pa.deficit", "Pb.deficit"):
            out.append(f"{label} {key} versions_positive={self.counts.get(key, 0)} peak_bytes={self.peaks.get(key, 0)}")
        for key in ("Pa.free", "Pb.free"):
            out.append(f"{label} {key} min_bytes={self.peaks.get(key, 0)}")
        out.append(f"{label} retire_records max={self.peaks.get('retire_records', 0)}")
        for key in sorted(self.first):
            out.append(f"{label} first {key} history={self.first[key]}")
        return out


CONFIGS = [
    Config(3, 0, 2, 2, 8),
    Config(3, 2, 2, 2, 8),
    Config(4, 2, 3, 2, 6),
    Config(2, 2, 2, 3, 8),
    Config(3, 0, 2, 2, 8, death_at_intent=True),
]
PREFIX_CREATES = 4


def exhaustive(cfg, depth):
    """前缀 = 建 PREFIX_CREATES 个 inode；之后每一步在全部合法操作里放开扫（建、对每个活号发意图、最后一批、回退到每个候选根、
    崩溃 m = 1、2），深度 depth，每一步之后判那一版。"""
    tally = Tally()
    root = Pool.initial(cfg, PREFIX_CREATES)
    stack = [(root, ())]
    while stack:
        pool, history = stack.pop()
        if len(history) == depth:
            continue
        for op in operations(pool):
            nxt = apply(pool, op)
            if nxt is None:
                continue
            h = history + (op,)
            tally.add(nxt, h)
            stack.append((nxt, h))
    return tally


def randomized(cfg, seed, runs, length, allow_rollback=True):
    """随机长历史；allow_rollback = False 是准入那一格的对照：不回退时 P-b 的缺口应当恒不为正。"""
    rng = random.Random(seed)
    tally = Tally()
    for _ in range(runs):
        pool = Pool.initial(cfg, PREFIX_CREATES)
        history = ()
        for _ in range(length):
            ops = [op for op in operations(pool) if allow_rollback or op[0] != "rb"]
            weights = [4 if op[0] == "create" else 2 if op[0] in ("final", "rb") else 1 for op in ops]
            nxt = None
            for _attempt in range(8):
                op = rng.choices(ops, weights)[0]
                nxt = apply(pool, op)
                if nxt is not None:
                    break
            if nxt is None:
                break
            history = history + (op,)
            pool = nxt
            tally.add(pool, history)
    return tally


def describe(pool):
    r = judge(pool)
    v = pool.cur
    return (f"txg={v.txg} inst={v.inst} leaves={[(l[0], l[1], l[2]) for l in v.leaves]} "
            f"retire_records={[(r_[0], r_[1]) for r_ in retire_records(v)]} lineage={list(v.lineage)} "
            f"C1={r['C1']} C1L={r['C1L']} C4={r['C4']} C4L={r['C4L']} "
            f"A0.res={r['A0.res']} A0.fd={r['A0.fd']} A1.fd={r['A1.fd']} A2.fd={r['A2.fd']} "
            f"same_ident_max={r['same_ident_max']} Pa.free_slots={r['Pa.free'] / SLOT:g} Pb.free_slots={r['Pb.free'] / SLOT:g} "
            f"Pa.deficit_slots={r['Pa.deficit'] / SLOT:g} Pb.deficit_slots={r['Pb.deficit'] / SLOT:g}")


def demo_loop(rounds):
    """K2 / K4 / K5：同一个身份被退役很多次。建 4 个（cap 3：叶 [1,2,3]、[4]），之后每一轮：对 4 发意图、最后一批（[4] 删空、
    写退役记录）、回退到这一轮开头那一版（4 回来、退役记录留在照抄 cur 的退役树里）。没有一步是准入式子判的。"""
    cfg = CONFIGS[0]
    pool = Pool.initial(cfg, PREFIX_CREATES)
    lines = [f"demo loop start {describe(pool)}"]
    for k in range(1, rounds + 1):
        start = pool.cur
        pool = pool.intent(4).final()
        lines.append(f"demo loop round={k} after_final {describe(pool)}")
        pool = pool.rollback(start)
        lines.append(f"demo loop round={k} after_rollback {describe(pool)}")
    return lines


def demo_first_container():
    """K4：建第 5 个之后可用正好为 0（准入判过、用户写满），再 unlink 那个刚分裂出来的单条叶：第一条退役记录要开第一片容器与根。"""
    cfg = CONFIGS[0]
    pool = Pool.initial(cfg, 7)  # cap 3：[1,2,3]、[4,5,6]、[7]，建 7 那一次是准入判过的写入类发布
    lines = [f"demo first_container judged {describe(pool)}"]
    pool = pool.intent(7).final()
    lines.append(f"demo first_container after_unlink {describe(pool)}")
    return lines


def demo_death_at_intent():
    """K2 死亡 txg 取哪一次：同一段历史，死亡 txg 取摘掉那次（主写法）与取 unlink 意图那次（对照）。建 4 个（叶 [1,2,3]、[4]），
    对 4 发意图（txg 7）、最后一批（txg 8，[4] 删空），再回退到意图那一版（txg 7：叶 [4] 还在，4 在队列里）。"""
    lines = []
    for cfg in (CONFIGS[0], CONFIGS[4]):
        pool = Pool.initial(cfg, PREFIX_CREATES).intent(4)
        intent_version = pool.cur
        pool = pool.final().rollback(intent_version)
        lines.append(f"demo death_at_intent {cfg.name()} {describe(pool)}")
    return lines


def demo_controls():
    """阳性对照：读法 A0 在扫描时丢一条退役记录（摘掉那片退役容器的全部版本）⇒ 复活；把被吸收的身份塞回叶序 ⇒ C1L 红。"""
    cfg = CONFIGS[1]
    pool = Pool.initial(cfg, 6)  # cap 3：[1,2,3]、[4,5,6]
    pool = pool.intent(2).final().intent(3).final()  # θ = 2：[1] 不足 ⇒ 1 + 3 > 3 不能吸收 ⇒ 从右借 1 条（N1，右片保留身份）
    lines = [f"demo controls base {describe(pool)}"]
    pool = pool.intent(4).final().intent(5).final()
    lines.append(f"demo controls merged {describe(pool)}")
    damaged = pool.copy()
    damaged.retire_units = {}
    lines.append(f"demo controls pc_reader_drops_retire_units A0.res={judge(damaged)['A0.res']}")
    v = pool.cur
    retired = retire_records(v)
    if retired:
        ident = retired[-1][0]
        bad = pool.copy()
        bad.cur = Version(v.txg, v.inst, v.parent, tuple(sorted(v.leaves + ((ident[0], ident[1], (max(r for l in v.leaves for r in l[2]) + 1,)),),
                                                                 key=lambda l: min(l[2]))),
                          v.queue, v.retire, v.lineage, v.wm, v.rows, v.judged)
        r = judge_rules_only(bad)
        lines.append(f"demo controls pc_checker_retired_identity_put_back C1={r['C1']} C1L={r['C1L']}")
    return lines


def judge_rules_only(pool):
    version = pool.cur
    lineage = lineage_set(version)
    truth = {leaf_ident(leaf) for leaf in version.leaves}
    records = retire_records(version)
    return {"C1": int(any(r[0] in truth for r in records)),
            "C1L": int(any(r[0] in truth for r in records if r[1] in lineage))}


def main(argv):
    if argv[1] == "sweep":
        index, depth = int(argv[2]), int(argv[3])
        cfg = CONFIGS[index]
        for line in exhaustive(cfg, depth).lines(f"sweep {cfg.name()} depth={depth}"):
            print(line)
    elif argv[1] == "random":
        index, seed, runs, length = (int(a) for a in argv[2:6])
        cfg = CONFIGS[index]
        for line in randomized(cfg, seed, runs, length).lines(f"random {cfg.name()} seed={seed} runs={runs} len={length}"):
            print(line)
    elif argv[1] == "random-norb":
        index, seed, runs, length = (int(a) for a in argv[2:6])
        cfg = CONFIGS[index]
        for line in randomized(cfg, seed, runs, length, allow_rollback=False).lines(
                f"random-norb {cfg.name()} seed={seed} runs={runs} len={length}"):
            print(line)
    elif argv[1] == "demo":
        for line in demo_loop(int(argv[2])) + demo_first_container() + demo_death_at_intent() + demo_controls():
            print(line)
    else:
        raise SystemExit("用法：retire_model.py sweep <配置> <深度> | random <配置> <种子> <段数> <长度> | demo <轮数>")


if __name__ == "__main__":
    main(sys.argv)
