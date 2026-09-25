#!/usr/bin/env python3
"""Z17：按条款独立推四个测试里钉的数，不读实现、不跑实现。
依据（kb 快照 /tmp/claude-1000/m2-final-code-r3/kb-snapshot/.claude/kb/decisions/）：
  D3 已定项 10（03-空间分配.md 第 200 行）：段长 64；段起点 = 单元区内最低的 64 槽对齐全空段；bump 指针只住内存、挂载后新开一段；
    用户数据取最低的起点 32768 对齐、两槽都空、不在聚簇段里的；提交内生块从开放段 bump；次序：实例表最前（尾片先），其余树 ID 升序、
    树内先叶后根、同层按 key 升序，中央映射树倒数第二，树表最末；码 3 容器取 32768 对齐的两槽。
  D8 已定项 14（08-核心索引结构.md 第 385、390、394、395 行）：分配记录树叶罩 [k×W,(k+1)×W)，W = 812，扇出 169，
    根层取最小的 R ≥ 1 使 Σ⌈槽数 ÷ (W×169^(R−1))⌉ ≤ 169；没有记录的一段不写；只重写内容变了的叶与它们的祖先（COW）。
  D3 已定项 7（释放 = 改写成已释放 + 释放代，条目不删；回收之前槽照占着）。
两块盘对称（同槽落两份），所以只模一块盘的槽，叶与内部节点按盘各算一份。
「全空段」取「段里没有被占着的槽（已分配或已释放未回收或被隔离）」；另按条款字面「没有未释放分配记录」算一遍，两种读法在这几段历史上给的段不同就报出来。
"""
import sys

W, F = 812, 169
UNIT_START = 50176
SEG = 64
DEVICE_SLOTS = (4 << 30) // 16384          # 262144
UNIT_AREA = DEVICE_SLOTS - UNIT_START       # 211968
TREE_ID = {"extent": 11, "inode": 12, "alloc": 13, "accounting": 14, "mapping": 15}

def root_level(slots_per_device, devices=2):
    r = 1
    while True:
        span = W * F ** (r - 1)
        if devices * -(-slots_per_device // span) <= F:
            return r
        r += 1

ROOT_LEVEL = root_level(DEVICE_SLOTS)

class Pool:
    def __init__(self):
        self.records = {}      # slot -> [span, gen, released]
        self.reclaimed = set()
        self.isolated = set()
        self.open_seg = None
        self.cursor = None
        self.cluster = set()
        self.seg_reading_mismatch = []
    def clone(self):
        p = Pool()
        p.records = {k: list(v) for k, v in self.records.items()}
        p.reclaimed = set(self.reclaimed); p.isolated = set(self.isolated)
        p.open_seg, p.cursor, p.cluster = self.open_seg, self.cursor, set(self.cluster)
        p.seg_reading_mismatch = self.seg_reading_mismatch
        return p
    def occupied(self, s):
        for start, (span, gen, rel) in self.records.items():
            if start <= s < start + span and start not in self.reclaimed:
                return True
        return s in self.isolated
    def unreleased(self, s):
        for start, (span, gen, rel) in self.records.items():
            if start <= s < start + span and not rel:
                return True
        return False
    def record(self, slot, span, gen):
        for start in [k for k in self.records if k in self.reclaimed and slot <= k < slot + span]:
            del self.records[start]; self.reclaimed.discard(start)
        self.records[slot] = [span, gen, False]
    def release(self, slot, gen):
        r = self.records[slot]; assert not r[2]; r[2] = True; r[1] = gen
    def lowest_empty_segment(self):
        seg = UNIT_START
        while seg + SEG <= UNIT_START + UNIT_AREA:
            a = not any(self.occupied(s) for s in range(seg, seg + SEG))
            b = not any(self.unreleased(s) for s in range(seg, seg + SEG))
            if a or b:
                if a != b:
                    self.seg_reading_mismatch.append(seg)
                if a:
                    return seg
            seg += SEG
        raise RuntimeError("no empty segment")
    def user_data(self, gen):
        s = UNIT_START
        while True:
            in_cluster = any(c <= s < c + SEG for c in self.cluster)
            if not in_cluster and not self.occupied(s) and not self.occupied(s + 1):
                self.record(s, 2, gen); return s
            s += 2
    def commit_generated(self, span, gen):
        if self.open_seg is not None:
            s = self.cursor
            if span == 2 and s % 2: s += 1
            while s + span <= self.open_seg + SEG and any(self.occupied(x) for x in range(s, s + span)):
                s += span
            if s + span <= self.open_seg + SEG:
                self.cursor = s + span; self.record(s, span, gen); return s
        seg = self.lowest_empty_segment()
        self.open_seg, self.cursor = seg, seg; self.cluster.add(seg)
        return self.commit_generated(span, gen)
    def remount(self):
        self.open_seg = None; self.cursor = None
    def leaf_contents(self):
        leaves = {}
        for start, v in self.records.items():
            leaves.setdefault(start // W, []).append((start, tuple(v)))
        return {k: sorted(v) for k, v in leaves.items()}

def nodes_of_leaves(leaf_indices):
    nodes = []
    for dev in (0, 1):
        for leaf in sorted(leaf_indices):
            nodes.append((0, dev, leaf))
    for level in range(1, ROOT_LEVEL):
        for dev in (0, 1):
            for idx in sorted({leaf // F ** level for leaf in leaf_indices}):
                nodes.append((level, dev, idx))
    nodes.sort()
    nodes.append("root")
    return nodes

def publish(pool, prev_roles, txg, file_roles, release_slots, instance_table=False, first_version=False):
    """file_roles: [(名字, 跨度, 种类)] 种类 = 'data' 用户数据 / 'commit' 提交内生块；按 bump 次序给（实例表、extent、inode 在前）。
    返回 (这次重写的角色与落点, 新的 prev_roles)。分配记录树重写哪几个节点走固定点。"""
    before = pool.leaf_contents()
    changed_nodes = []
    for _ in range(64):
        trial = pool.clone()
        for s in release_slots:
            trial.release(s, txg)
        roles = list(file_roles) + [(("alloc", n), 1, "commit") for n in changed_nodes] + \
                [("accounting", 1, "commit"), ("mapping", 1, "commit"), ("tree_table", 1, "commit")]
        placed = []
        for name, span, kind in roles:
            slot = trial.user_data(txg) if kind == "data" else trial.commit_generated(span, txg)
            placed.append((name, span, slot))
        after = trial.leaf_contents()
        changed_leaves = {k for k in set(before) | set(after) if before.get(k) != after.get(k)}
        new_nodes = nodes_of_leaves(changed_leaves) if changed_leaves else []
        if new_nodes == changed_nodes:
            pool.__dict__.update(trial.__dict__)
            return placed
        changed_nodes = sorted(set(changed_nodes) | set(new_nodes), key=lambda n: (1, 0, 0, 0) if n == "root" else (0,) + n)
    raise RuntimeError("fixed point did not settle")

def slots_of(placed):
    return [slot for _, _, slot in placed]

def spans(placed):
    return sum(span for _, span, _ in placed)

def main():
    out = []
    # ── 第一个事务（A，txg 3）──
    pool = Pool()
    pool.record(UNIT_START, 2, 0)        # mkfs 实例表（码 3，两槽）
    pool.record(UNIT_START + 2, 1, 0)    # mkfs 树表
    file_roles = [("data", 2, "data"), ("extent_root", 1, "commit"), ("inode_leaf", 2, "commit"), ("inode_root", 1, "commit")]
    A = publish(pool, None, 3, file_roles, release_slots=[UNIT_START + 2])
    out.append(("根层（4 GiB × 2）", ROOT_LEVEL))
    out.append(("A 单元数", len(A)))
    out.append(("A 落点", slots_of(A)))
    out.append(("A 分配记录树节点", [n for n, _, _ in A if isinstance(n, tuple) and n[0] == "alloc"]))
    out.append(("A 树表落点", dict((n, s) for n, _, s in A)["tree_table"]))
    out.append(("A 单元写次数（两盘）", 2 * len(A)))
    # ── B：同一进程覆盖写（txg 4）──
    poolB = pool.clone()
    B = publish(poolB, A, 4, file_roles, release_slots=[s for _, _, s in A])
    out.append(("B 落点", [(n if not isinstance(n, tuple) else n[1], s) for n, _, s in B]))
    out.append(("B 之后分配记录条数（两盘）", 2 * len(poolB.records)))
    allocated = sum(v[0] for v in poolB.records.values())
    deferred = sum(v[0] for v in poolB.records.values() if v[2])
    out.append(("B 之后已分配槽 / defer 槽 / 空闲槽", (allocated, deferred, UNIT_AREA - allocated)))
    out.append(("B 之后空闲字节", (UNIT_AREA - allocated) * 16384))
    out.append(("B 映射条目（码 1 一条 + 码 2 / 码 3：extent 根、inode 叶、inode 根、记账根、分配记录树节点）",
                1 + 3 + 1 + sum(1 for n, _, _ in B if isinstance(n, tuple))))
    # ── 一次建 13280 个 inode（57 片叶容器）──
    poolI = pool.clone()
    containers = [(("inode_leaf", k), 2, "commit") for k in range(57)]
    I = publish(poolI, A, 4, containers + [("inode_root", 1, "commit")],
                release_slots=[dict((n, s) for n, _, s in A)[x] for x in ("inode_leaf", "inode_root")] +
                              [s for n, _, s in A if isinstance(n, tuple) or n in ("accounting", "mapping", "tree_table")])
    alloc_nodes_I = [n for n, _, _ in I if isinstance(n, tuple) and n[0] == "alloc"]
    out.append(("建 13280 个 inode：分配记录树重写节点", alloc_nodes_I))
    out.append(("建 13280 个 inode：非容器角色数 / 总角色数", (len(I) - 57, len(I))))
    out.append(("建 13280 个 inode：叶容器落点首末", (slots_of(I)[0], slots_of(I)[56])))
    # ── 回退那一段：A、B（实例 1）、重开取号 2、写行（txg 5）、暖机两次（6、7）、C（txg 8），回退到 A（实例 3，写行 txg 9）──
    def history_to_c():
        p = pool.clone()
        b = publish(p, A, 4, file_roles, release_slots=[s for _, _, s in A])
        p.remount()
        fixed = lambda placed: [s for n, _, s in placed if isinstance(n, tuple) or n in ("accounting", "mapping", "tree_table")]
        row = publish(p, b, 5, [("instance_table", 2, "commit")], release_slots=[UNIT_START] + fixed(b))
        w1 = publish(p, row, 6, [], release_slots=fixed(row))
        w2 = publish(p, w1, 7, [], release_slots=fixed(w1))
        c = publish(p, w2, 8, file_roles, release_slots=[s for n, _, s in b if n in ("data", "extent_root", "inode_leaf", "inode_root")] + fixed(w2))
        return p, b, row, w1, w2, c
    p, b, row, w1, w2, c = history_to_c()
    out.append(("B / 写行 / 暖机 / 暖机 / C 各占槽", [spans(x) for x in (b, row, w1, w2, c)]))
    # 回退到 A：分配器从 A 那一版的账重建（A 之后的槽都缺席 = 空闲），被抛弃根独占的槽影子账开着时隔离。
    abandoned = set()
    for placed in (b, row, w1, w2, c):
        for _, span, slot in placed:
            abandoned.update(range(slot, slot + span))
    out.append(("被抛弃根独占的槽（逐盘）", len(abandoned)))
    only_c = set()
    for _, span, slot in c:
        only_c.update(range(slot, slot + span))
    out.append(("去掉只被 C 引用的之后", len(abandoned - only_c)))
    fixed_A = [s for n, _, s in A if isinstance(n, tuple) or n in ("accounting", "mapping", "tree_table")]
    for shadow in (True, False):
        q = pool.clone(); q.remount()
        if shadow:
            q.isolated = set(abandoned)
        d = publish(q, A, 9, [("instance_table", 2, "commit")], release_slots=[UNIT_START] + fixed_A)
        out.append((f"回退写行 D（影子账{'开' if shadow else '关'}）落点与节点",
                    [(n if not isinstance(n, tuple) else n[1], s) for n, _, s in d]))
        out.append((f"回退写行 D（影子账{'开' if shadow else '关'}）占槽", spans(d)))
    out.append(("两种「全空段」读法分歧的段起点", sorted(set(pool.seg_reading_mismatch))))
    for k, v in out:
        print(f"{k}\t{v}")

if __name__ == "__main__":
    main()
