#!/usr/bin/env python3
"""m4-trunc-r2 正推腿的小模型：单元级纯算术，不读 crates/ 的代码、不起 cargo。

只模拟「一个对象（一个 inode、一个对象出生代）的数据单元在盘上的出生 txg」与「扫描重建按各臂的读法认领哪些」，
和「遍历方向（真值）认哪些」逐个偏移比。真值 = 现行那一版里存在、且在 size 之内的单元，出生 txg 是现行那一版的。

对应 crates/ 里的规矩（读码得来，模型里按这些写，不是 crates/ 的复制）：
- 一个文件要几个单元：nunits(size) = max(1, ceil(size / 净荷))（write_request_split.rs 的 data_unit_count_of_a_sequential_write）。
- 截断意图那次重写尾单元 k 的判法：truncate.rs tail_data_unit_rewrite_of_a_truncation（k 存在且声明长度变了才重写）。
- 变长那次重写最后一个存在的单元：truncate.rs tail_data_unit_rewrite_of_an_extension。
- 扫描重建的读法 Z 加写序：rebuild.rs rebuild_index_tables 里 size 之内 + newest_birth_txg_below_each_data_unit。
- 整份覆盖写（W）：每个单元都用新 txg 重写，size = 新长度，从偏移 0 起。
- 搬迁（整理）不改出生 txg（D26 已定项 5），模型里没有这一步：它不改任何单元的出生 txg，所以不进历史。

净荷在模型里取 2 字节（P = 2），size 0..8 即 0 到 4 个单元，奇数 size 落在单元中间。
"""
import sys

P = 2
INF = 10 ** 9


def nunits(size):
    return max(1, -(-size // P))


def decl(i, size):
    return min(max(size - i * P, 0), P)


class St:
    __slots__ = ("size", "live", "disk", "field", "tombs", "t", "abandoned", "label")

    def copy(self):
        s = St()
        s.size = self.size
        s.live = dict(self.live)
        s.disk = {i: list(b) for i, b in self.disk.items()}
        s.field = self.field
        s.tombs = list(self.tombs)
        s.t = self.t
        s.abandoned = set(self.abandoned)
        s.label = self.label
        return s


def initial(size=8):
    s = St()
    s.t = 4
    s.size = size
    s.live = {i: 4 for i in range(nunits(size))}
    s.disk = {i: [4] for i in range(nunits(size))}
    s.field = None
    s.tombs = []
    s.abandoned = set()
    s.label = "W%d" % size
    return s


ARMS = {
    # 名字: order=写序那一条要不要, field=甲字段怎么盖章, wf=整份覆盖写怎么处理字段, tomb=乙墓碑什么时候写, tail=截断是否总重写尾单元
    "K6": dict(order=True, field=None, wf="carry", tomb=None, tail=False),
    "丙": dict(order=True, field=None, wf="carry", tomb=None, tail=True),
    "甲last": dict(order=True, field="last", wf="clear", tomb=None, tail=False),
    "甲rel清": dict(order=True, field="rel", wf="clear", tomb=None, tail=False),
    "甲rel留": dict(order=True, field="rel", wf="carry", tomb=None, tail=False),
    "甲rel覆盖也盖章": dict(order=True, field="rel", wf="stamp", tomb=None, tail=False),
    "甲rel覆盖也盖章无写序": dict(order=False, field="rel", wf="stamp", tomb=None, tail=False),
    "乙截断": dict(order=True, field=None, wf="carry", tomb="trunc", tail=False),
    "乙截断无写序": dict(order=False, field=None, wf="carry", tomb="trunc", tail=False),
    "乙全释放无写序": dict(order=False, field=None, wf="carry", tomb="all", tail=False),
}


def claim(s, arm):
    cur = {}
    for i, births in s.disk.items():
        bs = [b for b in births if b not in s.abandoned]
        if bs:
            cur[i] = max(bs)
    n = nunits(s.size)
    out = {}
    for i, b in cur.items():
        if i >= n:
            continue
        if arm["order"] and any(b2 > b for j, b2 in cur.items() if j < i):
            continue
        if arm["field"] and s.field is not None:
            ft, fk = s.field
            if i >= fk and b < ft:
                continue
        if arm["tomb"]:
            dead = False
            for (tt, a, bb) in s.tombs:
                if tt in s.abandoned:
                    continue
                if a <= i < bb and b < tt:
                    dead = True
                    break
            if dead:
                continue
        out[i] = b
    return out


def truth(s):
    n = nunits(s.size)
    return {i: b for i, b in s.live.items() if i < n}


def verdict(s, arm):
    c, t = claim(s, arm), truth(s)
    revived = sorted(i for i in c if i not in t or c[i] != t[i])
    lost = sorted(i for i in t if i not in c)
    return revived, lost


def op_write(s, arm, length):
    s = s.copy()
    s.t += 1
    n = nunits(length)
    released = any(i >= n for i in s.live)
    s.live = {i: s.t for i in range(n)}
    for i in range(n):
        s.disk.setdefault(i, []).append(s.t)
    s.size = length
    if arm["wf"] == "clear":
        s.field = None
    elif arm["wf"] == "stamp" and arm["field"]:
        if released:
            s.field = (s.t, n)
    if arm["tomb"] == "all" and released:
        s.tombs.append((s.t, n, INF))
    s.label = "W%d" % length
    return s, None


def op_size(s, arm, target):
    """改 size：N<S 截断（意图 + 批，同时返回意图后的中间态），N>S 变长。"""
    if target == s.size:
        return None, None
    s = s.copy()
    s.t += 1
    if target < s.size:
        K = nunits(target)
        k = K - 1
        rewrite = k in s.live and (arm["tail"] or decl(k, s.size) != decl(k, target))
        if rewrite:
            s.live[k] = s.t
            s.disk.setdefault(k, []).append(s.t)
        releases = any(i >= K for i in s.live)
        if arm["field"] == "last":
            s.field = (s.t, K)
        elif arm["field"] == "rel" and releases:
            s.field = (s.t, K)
        if arm["tomb"] in ("trunc", "all"):
            s.tombs.append((s.t, K, INF))
        s.size = target
        s.label = "S%d" % target
        mid = s.copy()
        mid.label = "S%d(意图后、批前)" % target
        s.live = {i: b for i, b in s.live.items() if i < K}
        s.t += 1  # 最后一批
        return s, mid
    if s.live:
        j = max(s.live)
        if decl(j, s.size) < decl(j, target):
            s.live[j] = s.t
            s.disk.setdefault(j, []).append(s.t)
    s.size = target
    s.label = "S%d" % target
    return s, None


def op_rollback(s, arm, hist, j):
    target = hist[j]
    s2 = s.copy()
    s2.t += 1
    s2.abandoned = set(target.abandoned) | set(range(target.t + 1, s2.t))  # 谱系：目标那一版的父链，加上这次回退抛弃的一段
    s2.size = target.size
    s2.live = dict(target.live)
    s2.field = target.field
    n = nunits(s2.size)
    if any(i >= n for i in s2.live):  # 回退到批前中间态：续做把挂着的单元放掉
        s2.live = {i: b for i, b in s2.live.items() if i < n}
        s2.t += 1
    s2.label = "R(%s)" % target.label
    return s2, None


def op_pwrite(s, arm, i):
    s = s.copy()
    s.t += 1
    s.live[i] = s.t
    s.disk.setdefault(i, []).append(s.t)
    s.size = max(s.size, (i + 1) * P)
    s.label = "PW%d" % i
    return s, None


def op_punch(s, arm, i):
    if i not in s.live:
        return None, None
    s = s.copy()
    s.t += 1
    del s.live[i]
    if arm["tomb"] == "all":
        s.tombs.append((s.t, i, i + 1))
    s.label = "PU%d" % i
    return s, None


def apply(kind, s, arm, hist, v):
    if kind == "W":
        return op_write(s, arm, v)
    if kind == "S":
        return op_size(s, arm, v)
    if kind == "R":
        return op_rollback(s, arm, hist, v)
    if kind == "PW":
        return op_pwrite(s, arm, v)
    if kind == "PU":
        return op_punch(s, arm, v)
    raise ValueError(kind)


def run_history(arm, steps, start=8):
    s = initial(start)
    hist = [s]
    rows = [(s.label,) + verdict(s, arm)]
    for kind, v in steps:
        s2, mid = apply(kind, s, arm, hist, v)
        if s2 is None:
            continue
        if mid is not None:
            rows.append((mid.label,) + verdict(mid, arm))
        s = s2
        hist.append(s)
        rows.append((s.label,) + verdict(s, arm))
    return rows


def search(arm, alphabet, depth, start=8, limit=3):
    found = []
    count = [0]
    visited = [0]

    def ops_here(s, hist):
        ops = []
        if "W" in alphabet:
            ops += [("W", L) for L in range(0, 9)]
        if "S" in alphabet:
            ops += [("S", N) for N in range(0, 9) if N != s.size]
        if "R" in alphabet:
            ops += [("R", j) for j in range(len(hist) - 1)]
        if "PW" in alphabet:
            ops += [("PW", i) for i in range(0, 4)]
        if "PU" in alphabet:
            ops += [("PU", i) for i in range(0, 4)]
        return ops

    def rec(s, hist, steps):
        visited[0] += 1
        if len(steps) == depth:
            return
        for kind, v in ops_here(s, hist):
            s2, mid = apply(kind, s, arm, hist, v)
            if s2 is None:
                continue
            new_steps = steps + [(kind, v)]
            bad = None
            for st in ([mid] if mid is not None else []) + [s2]:
                rv, lo = verdict(st, arm)
                if rv or lo:
                    bad = (st.label, rv, lo)
                    break
            if bad:
                count[0] += 1
                found.append((len(new_steps), new_steps, bad))
                continue
            rec(s2, hist + [s2], new_steps)

    rec(initial(start), [initial(start)], [])
    found.sort(key=lambda x: x[0])
    return count[0], visited[0], found[:limit]


def fmt_steps(steps):
    return " ".join("%s%s" % (k, v) for k, v in steps)


NAMED = [
    ("H1 截在单元边界再变长（题面）", [("S", 2), ("S", 7)]),
    ("H2 截到单元中间再变长", [("S", 3), ("S", 8)]),
    ("H3 截到 0 再变长", [("S", 0), ("S", 8)]),
    ("H4 截到 N1 → 变长 → 截到 N2>N1（落在洞里）→ 变长", [("S", 2), ("S", 5), ("S", 4), ("S", 8)]),
    ("H5 截断后整份覆盖写成更长、再截短、再变长", [("S", 2), ("W", 8), ("S", 2), ("S", 8)]),
    ("H5b 整份覆盖写变短再变长（无截断）", [("W", 4), ("S", 8)]),
    ("H6 截断中间态（意图后批前）", [("S", 2)]),
    ("H7 截断之后回退到截断之前那一版，再截再变长", [("S", 2), ("R", 0), ("S", 2), ("S", 8)]),
    ("H8 截深、变长、再截回刚才的边界、变长", [("S", 4), ("S", 8), ("S", 2), ("S", 8)]),
    ("H9 截到边界、变长、再截到更小的边界、变长", [("S", 6), ("S", 8), ("S", 2), ("S", 8)]),
]


def main():
    global ARMS
    mode = sys.argv[1] if len(sys.argv) > 1 else "all"
    depth = int(sys.argv[2]) if len(sys.argv) > 2 else 4
    if len(sys.argv) > 3:  # 第三个参数：只跑这些臂（逗号分隔）
        wanted = sys.argv[3].split(",")
        ARMS = {k: v for k, v in ARMS.items() if k in wanted}
    if mode in ("all", "named"):
        print("## 命名历史（起点：一个 4 单元的文件，净荷 2 字节，size 8；S<n> 改 size 到 n，W<n> 整份覆盖写成 n 字节，R<j> 回退到第 j 个状态）")
        for name, steps in NAMED:
            print("### %s：%s" % (name, fmt_steps(steps)))
            for an, arm in ARMS.items():
                rows = run_history(arm, steps)
                bad = [r for r in rows if r[1] or r[2]]
                if not bad:
                    print("  %-20s 全步相同" % an)
                else:
                    lab, rv, lo = bad[0]
                    print("  %-20s 第一处不同在 %s：多认领单元 %s，漏认领单元 %s" % (an, lab, rv, lo))
    if mode in ("all", "search"):
        print("## 穷举：字母表 W S R，深度 %d（起点同上；一条历史走到不同就记下、不再往下扩）" % depth)
        for an, arm in ARMS.items():
            count, visited, found = search(arm, {"W", "S", "R"}, depth)
            print("  %-20s 访问 %d 个状态，最短不同的历史共 %d 条" % (an, visited, count))
            for (n, steps, bad) in found:
                print("      例：%s  → %s 多认领 %s 漏认领 %s" % (fmt_steps(steps), bad[0], bad[1], bad[2]))
    if mode in ("all", "w5"):
        for title, alpha in (("加 pwrite（PW<i>：只重写第 i 个单元）", {"W", "S", "R", "PW"}),
                             ("加 pwrite 与 punch（PU<i>：只放掉第 i 个单元）", {"W", "S", "R", "PW", "PU"})):
            print("## W5 %s，深度 %d" % (title, depth))
            for an, arm in ARMS.items():
                count, visited, found = search(arm, alpha, depth)
                print("  %-20s 访问 %d 个状态，最短不同的历史共 %d 条" % (an, visited, count))
                for (n, steps, bad) in found:
                    print("      例：%s  → %s 多认领 %s 漏认领 %s" % (fmt_steps(steps), bad[0], bad[1], bad[2]))


if __name__ == "__main__":
    main()
