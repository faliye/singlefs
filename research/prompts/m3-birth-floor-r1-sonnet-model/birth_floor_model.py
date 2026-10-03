#!/usr/bin/env python3
"""诞生代号下界（I-3.5 ② 下界）与 I-3.5 ①：V1 各种写法在带管理员回退的合法历史上红不红、坏镜像上漏不漏。纯 Python 抽象模型，不读不跑 crates。

单元 = 嵌套元组 (类, 诞生代号, 子单元们)，相等的元组就是同一个落点。
  Ir/Il = inode 树根/叶，Xr/Xl = extent 树根/叶，d = 数据单元，fx = 固定点单元（树表、记账、分配记录树、中央映射根、实例表：每次发布都新写）。
根 = (txg, 操作, I 树根单元或 None, X 树根单元或 None)。引用 = fx(txg) + I、X 两棵子树。
操作（与 crates/ 今天对得上的）：
  W  覆盖写（transaction.rs publish_overwrite：数据、extent 叶根、inode 叶根全换新，改动计数 = txg 所以 inode 树必换）
  N  建 inode（publish_new_inodes：只换 inode 叶根，extent 树与数据不动）
  E  空发布（暖机、写行、推空：两棵树的根指针照抄）
  R(k) 管理员回退（mount.rs roll_back_by_a_forward_publish：两棵树的根指针取候选集第 k 条根的，固定点新写）
  F(x) 抬 F（上限 = 第 4 新的去重非空根的 txg，不足 4 个取最旧有效根）
  U    正常卸载（空发布 2 次，F 抬到卸载前现行那一版的 txg，不另判上限）
今天没有、里程碑三之后才有的（FUT 档才开）：
  C 建带数据的新文件（数据单元 + extent/inode 路径新）、M(k) 只改第 k 个文件的数据单元（别的文件的数据单元原样）
故障档：L(k) 候选集里第 k 条根的根槽读不出（盘掉了/槽坏了），它不再是候选。
根环容量 24（3 区域 × 8 槽，format lib.rs ROOT_RING_REGIONS、ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM）。
不模型：实例切换、被抛弃时间线、journal 记录、分配记录树里的释放记录。
"""
import sys, random, itertools
from functools import lru_cache
sys.dont_write_bytecode = True
RING = 24

def U(kind, birth, kids=()):
    return (kind, birth, tuple(kids))

@lru_cache(maxsize=None)
def subtree(u):
    out = {u}
    for k in u[2]:
        out |= subtree(k)
    return frozenset(out)

def refs(root):
    t, op, I, X = root
    s = {U("fx", t)}
    if I is not None:
        s |= subtree(I)
    if X is not None:
        s |= subtree(X)
    return frozenset(s)

def data_of(X):
    return list(X[2][0][2]) if X is not None else []

def build(txg, data, with_I=True, with_X=True, old_I=None, old_X=None):
    Xl = U("Xl", txg, data)
    X = U("Xr", txg, [Xl]) if with_X else old_X
    Il = U("Il", txg)
    I = U("Ir", txg, [Il]) if with_I else old_I
    return I, X

class H:
    """一段历史的状态。"""
    __slots__ = ("roots", "floor", "txg", "ops", "lost")
    def __init__(self, roots, floor, txg, ops, lost):
        self.roots, self.floor, self.txg, self.ops, self.lost = roots, floor, txg, ops, lost

def start():
    roots = ((0, "mkfs", None, None), (1, "warm", None, None), (2, "warm", None, None))
    I, X = build(3, [U("d", 3)])
    roots = roots + ((3, "first", I, X),)
    return H(roots, 0, 3, (), frozenset())

def candidates(h):
    ring = h.roots[-RING:]
    newest = h.roots[-1]
    return [r for r in ring if (r[0] >= h.floor and r[0] not in h.lost) or r is newest]

def valid_for_ceiling(h):
    return [r for r in h.roots[-RING:] if r[0] >= h.floor and r[0] not in h.lost]

def ceiling(h):
    valid = sorted(valid_for_ceiling(h), key=lambda r: r[0])
    if not valid:
        return None
    nonempty, prev_state = [], None
    for r in valid:
        state = (r[2], r[3])
        has_file = r[2] is not None or r[3] is not None
        if has_file and state != prev_state:
            nonempty.append(r)
        prev_state = state
    counted, seen = [], []
    for r in reversed(nonempty):
        state = (r[2], r[3])
        if state in seen:
            continue
        seen.append(state)
        counted.append(r[0])
    if len(counted) >= 4:
        return counted[3]
    return valid[0][0]

def step(h, op, fut=False, fault=False):
    k = op[0]
    cand = candidates(h)
    cur = h.roots[-1]
    if k == "F":
        c = ceiling(h)
        if c is None or not (h.floor < op[1] <= c):
            return None
        return H(h.roots, op[1], h.txg, h.ops + (op,), h.lost)
    if k == "L":
        if not fault:
            return None
        files = [r for r in cand if r is not cur]
        if op[1] >= len(files):
            return None
        return H(h.roots, h.floor, h.txg, h.ops + (op,), h.lost | {files[op[1]][0]})
    if k == "U":
        t = h.txg
        roots = h.roots + ((t + 1, "E", cur[2], cur[3]), (t + 2, "E", cur[2], cur[3]))
        return H(roots, max(h.floor, t), t + 2, h.ops + (op,), h.lost)
    t = h.txg + 1
    I, X = cur[2], cur[3]
    if k == "W":
        d = data_of(X)
        d[0] = U("d", t)
        I2, X2 = build(t, d)
    elif k == "N":
        _, X2 = I, X
        I2, _ = build(t, data_of(X), with_X=False, old_X=X)
    elif k == "E":
        I2, X2 = I, X
    elif k == "R":
        withfile = [r for r in cand if r[2] is not None]
        if op[1] >= len(withfile):
            return None
        tgt = withfile[op[1]]
        I2, X2 = tgt[2], tgt[3]
    elif k == "C" and fut:
        d = data_of(X) + [U("d", t)]
        I2, X2 = build(t, d)
    elif k == "M" and fut:
        d = data_of(X)
        if op[1] >= len(d):
            return None
        d[op[1]] = U("d", t)
        I2, X2 = build(t, d)
    else:
        return None
    return H(h.roots + ((t, k, I2, X2),), h.floor, t, h.ops + (op,), h.lost)

# ---------- 判定 ----------
def inherits(first, prev, kind, strict):
    """first 的 kind 树（I 或 X）根指针是「从候选集之外继承来的」：与 prev 的同种树根不同，且诞生代号 ≤（strict 时 <）prev.txg。"""
    idx = 2 if kind == "I" else 3
    p, q = first[idx], prev[idx]
    if p is None or p == q:
        return False
    return p[1] < prev[0] if strict else p[1] <= prev[0]

def analyse(h, guard=False):
    """返回 dict：各写法下限红的 (单元, R_first, R_prev) 集合，以及 ① 的结果。"""
    cand = sorted(candidates(h), key=lambda r: r[0])
    rs = [refs(r) for r in cand]
    units = set().union(*rs)
    out = {k: [] for k in ("plain", "opus", "both", "scoped", "strict", "strict_guard")}
    classes = []
    for b in units:
        pos = [i for i, s in enumerate(rs) if b in s]
        f = pos[0]
        if f == 0:
            continue
        first, prev = cand[f], cand[f - 1]
        if b[1] > prev[0]:
            continue
        out["plain"].append((b, first[0], prev[0]))
        for strict, name in ((False, "scoped"), (True, "strict")):
            ex = False
            for kind in ("I", "X"):
                if inherits(first, prev, kind, strict):
                    root = first[2] if kind == "I" else first[3]
                    if b in subtree(root):
                        ex = True
            if not ex:
                out[name].append((b, first[0], prev[0]))
                if name == "strict" and not (guard and prev[0] + 1 != first[0]):
                    pass
        if not inherits(first, prev, "I", False):
            out["opus"].append((b, first[0], prev[0]))
        if not (inherits(first, prev, "I", False) and inherits(first, prev, "X", False)):
            out["both"].append((b, first[0], prev[0]))
        if prev[0] + 1 == first[0] and (b, first[0], prev[0]) in out["strict"]:
            out["strict_guard"].append((b, first[0], prev[0]))
        classes.append((b, first, prev,
                        first[1] == "R",
                        inherits(first, prev, "I", False),
                        inherits(first, prev, "X", False)))
    # ① 连续无洞：plain 与「洞的右缘根是继承根」两种
    holes_plain, holes_relaxed = [], []
    for b in units:
        pos = [i for i, s in enumerate(rs) if b in s]
        for a, c in zip(pos, pos[1:]):
            if c - a > 1:
                holes_plain.append((b, cand[a][0], cand[c][0]))
                right, pred = cand[c], cand[c - 1]
                ex = False
                for kind in ("I", "X"):
                    if inherits(right, pred, kind, True):
                        root = right[2] if kind == "I" else right[3]
                        if b in subtree(root):
                            ex = True
                if not ex:
                    holes_relaxed.append((b, cand[a][0], cand[c][0]))
    out["hole_plain"] = holes_plain
    out["hole_relaxed"] = holes_relaxed
    out["classes"] = classes
    return out

ALPHABETS = {
    "HARNESS": [("W",), ("E",)] + [("R", k) for k in range(4)] + [("F", x) for x in range(4, 12)] + [("U",)],
    "API": [("W",), ("E",), ("N",)] + [("R", k) for k in range(4)] + [("F", x) for x in range(4, 12)] + [("U",)],
    "FUT": [("W",), ("E",), ("N",), ("C",)] + [("M", k) for k in range(2)] + [("R", k) for k in range(4)] + [("F", x) for x in range(4, 12)],
}

def dfs(alpha, depth, fut=False, fault=False):
    stats = {"histories": 0}
    keys = ("plain", "opus", "both", "scoped", "strict", "strict_guard", "hole_plain", "hole_relaxed")
    for k in keys:
        stats[k] = 0
    cls = {"A_whole": 0, "B_partial": 0, "C_none": 0, "D_nonrollback_first": 0}
    first_example = {}
    def rec(h, d):
        if d > 0:
            stats["histories"] += 1
            a = analyse(h)
            for k in keys:
                if a[k]:
                    stats[k] += 1
                    first_example.setdefault(k, (h.ops, a[k][:2]))
            for (b, first, prev, is_r, ii, ix) in a["classes"]:
                if is_r and ii and ix:
                    cls["A_whole"] += 1
                elif is_r and (ii or ix):
                    cls["B_partial"] += 1
                elif is_r:
                    cls["C_none"] += 1
                else:
                    cls["D_nonrollback_first"] += 1
        if d == depth:
            return
        for op in alpha:
            n = step(h, op, fut, fault)
            if n is not None:
                rec(n, d + 1)
    rec(start(), 0)
    return stats, cls, first_example

if __name__ == "__main__":
    name, depth = sys.argv[1], int(sys.argv[2])
    fut = name == "FUT"
    stats, cls, ex = dfs(ALPHABETS[name], depth, fut)
    print(f"字母表 {name}，深度 {depth}，不含故障")
    print("合法历史（前缀）条数:", stats["histories"])
    for k in ("plain", "opus", "both", "scoped", "strict", "strict_guard", "hole_plain", "hole_relaxed"):
        print(f"  {k:13s} 红的历史 {stats[k]}")
    print("  plain 下界红的单元按 R_first 归类:", cls)
    for k, v in ex.items():
        print("  首例", k, v)
