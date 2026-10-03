#!/usr/bin/env python3
"""诞生代号下界（I-3.5 ② 下界）与 I-3.5 ① 的出现规则模型 v2。纯 Python 抽象模型，不读不跑 crates；与前任落盘的 birth_floor_model.py 同族，
差别：回退目标按「从旧数 / 从新数」都枚举、加了实例表单元与挂载（K）、抛弃最新根（X）、故障丢根槽（L）、随机长历史（过根环）、坏镜像变异层。

单元 = (类, 诞生代号, 子单元们)，相等的元组就是同一个落点。类：Ir/Il inode 树根/叶，Xr/Xl extent 树根/叶，d 数据，it 实例表，fx 固定点（树表/记账/分配记录树/中央映射根：每次发布新写）。
根 = (txg, 操作, I 树根或 None, X 树根或 None, 实例表单元)。引用 = fx(txg) + I、X 两棵子树 + 实例表单元。
操作（对得上 crates/ 今天的）：W 覆盖写 publish_overwrite；N 建 inode publish_new_inodes（只换 inode 树，extent 与数据照抄）；E 空发布；
  R 回退 roll_back_by_a_forward_publish（inode、extent 两棵树的根指针取目标的，实例表照现行，固定点新写）；F 抬 F；U 正常卸载；
  K 可写挂载（写行：实例表新写，之后 2 次暖机空发布；条数是抽象，不是实现）；X 崩溃恢复抛弃最新根（根留在环里、不在候选集，txg 计数不回退）；
  C/M 里程碑三之后才有的（建带数据的新文件 / 只改第 k 个文件的数据）。L(k) 故障：候选集里第 k 条非现行根的根槽读不出。
根环容量 24（format lib.rs ROOT_RING_REGIONS × ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM）。不模型：journal 记录、分配记录树里的释放、中央映射树、两块盘各自的根。
"""
import sys, random
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

@lru_cache(maxsize=None)
def refs(root):
    t, op, I, X, IT = root
    s = {U("fx", t)}
    if IT is not None:
        s.add(IT)
    for p in (I, X):
        if p is not None:
            s |= subtree(p)
    return frozenset(s)

def data_of(X):
    return list(X[2][0][2]) if X is not None else []

def build(txg, data):
    # r3：inode 树两片叶（Ila 低号叶、Ilb 高号叶），对得上 J1-a 那段历史（建 300 个 inode 之后两片叶）
    return U("Ir", txg, [U("Ila", txg), U("Ilb", txg)]), U("Xr", txg, [U("Xl", txg, data)])

class H:
    __slots__ = ("roots", "floor", "txg", "ops", "lost", "ab")
    def __init__(self, roots, floor, txg, ops, lost, ab):
        self.roots, self.floor, self.txg, self.ops, self.lost, self.ab = roots, floor, txg, ops, lost, ab

def start():
    it0 = U("it", 0)
    I, X = build(3, [U("d", 3)])
    roots = ((0, "mkfs", None, None, it0), (1, "warm", None, None, it0), (2, "warm", None, None, it0), (3, "first", I, X, it0))
    return H(roots, 0, 3, (), frozenset(), frozenset())

def cur_root(h):
    for r in reversed(h.roots):
        if r[0] not in h.ab:
            return r

def candidates(h):
    cur = cur_root(h)
    return [r for r in h.roots[-RING:] if r[0] not in h.ab and ((r[0] >= h.floor and r[0] not in h.lost) or r is cur)]

def valid(h):
    return [r for r in h.roots[-RING:] if r[0] not in h.ab and r[0] >= h.floor and r[0] not in h.lost]

def ceiling(h):
    v = sorted(valid(h), key=lambda r: r[0])
    if not v:
        return None
    nonempty, prev = [], None
    for r in v:
        st = (r[2], r[3])
        if (r[2] is not None or r[3] is not None) and st != prev:
            nonempty.append(r)
        prev = st
    counted, seen = [], []
    for r in reversed(nonempty):
        st = (r[2], r[3])
        if st in seen:
            continue
        seen.append(st)
        counted.append(r[0])
    return counted[3] if len(counted) >= 4 else v[0][0]

def pick_target(h, sel):
    cur = cur_root(h)
    lst = sorted([r for r in candidates(h) if r[2] is not None and (r[2], r[3]) != (cur[2], cur[3])], key=lambda r: r[0])
    kind, k = sel
    if kind == "o":
        return lst[k] if k < len(lst) else None
    return lst[-1 - (k - 1)] if k - 1 < len(lst) else None

def step(h, op, fut=False, fault=False):
    k = op[0]
    cur = cur_root(h)
    if k == "F":
        c = ceiling(h)
        if c is None or not (h.floor < op[1] <= c):
            return None
        return H(h.roots, op[1], h.txg, h.ops + (op,), h.lost, h.ab)
    if k == "L":
        if not fault:
            return None
        files = [r for r in candidates(h) if r is not cur]
        if op[1] >= len(files):
            return None
        return H(h.roots, h.floor, h.txg, h.ops + (op,), h.lost | {files[op[1]][0]}, h.ab)
    if k == "X":
        if not fault or len(candidates(h)) < 2:
            return None
        prev = [r for r in candidates(h) if r is not cur][-1]
        if prev[2] is None or h.floor > prev[0]:
            return None
        return H(h.roots, h.floor, h.txg, h.ops + (op,), h.lost, h.ab | {cur[0]})
    t = h.txg + 1
    I, X, IT = cur[2], cur[3], cur[4]
    if k == "U":
        roots = h.roots + ((t, "E", I, X, IT), (t + 1, "E", I, X, IT))
        return H(roots, max(h.floor, cur[0]), t + 1, h.ops + (op,), h.lost, h.ab)
    if k == "K":
        it = U("it", t)
        roots = h.roots + ((t, "K", I, X, it), (t + 1, "E", I, X, it), (t + 2, "E", I, X, it))
        return H(roots, h.floor, t + 2, h.ops + (op,), h.lost, h.ab)
    if k == "W":
        # 覆盖写：低号叶重写（inode 记录 mtime / 改动计数变），高号叶照抄上一版；extent 树与数据单元新写
        d = data_of(X); d[0] = U("d", t); _, X2 = build(t, d)
        I2 = U("Ir", t, [U("Ila", t), I[2][1]])
    elif k == "N":
        # 建 inode：新号落在高号叶，高号叶重写，低号叶照抄；extent 树与数据单元不动
        I2, X2 = U("Ir", t, [I[2][0], U("Ilb", t)]), X
    elif k == "E":
        I2, X2 = I, X
    elif k == "R":
        tgt = pick_target(h, op[1])
        if tgt is None:
            return None
        I2, X2 = tgt[2], tgt[3]
    elif k == "C" and fut:
        I2, X2 = build(t, data_of(X) + [U("d", t)])
    elif k == "M" and fut:
        d = data_of(X)
        if op[1] >= len(d):
            return None
        d[op[1]] = U("d", t); I2, X2 = build(t, d)
    else:
        return None
    return H(h.roots + ((t, k, I2, X2, IT),), h.floor, t, h.ops + (op,), h.lost, h.ab)

# ---------- 判定 ----------
VARIANTS = ("plain", "opusI", "both", "sc_le", "sc_lt")

def analyse(h, bo=None, po=None, extra=None):
    """候选集按 txg 升序。对每条非最古的候选根 R_j 与每个「出现」单元 b（b 在 R_j 引用里、不在 R_{j-1} 里）：
    第一次出现（更早的候选根都不引用）= ② 下界的对象；再次出现（洞的右缘）= ① 的对象。
    bo: 单元 -> 改过的诞生代号（头里的数）；po: (根 txg, 'I'|'X') -> 改过的树表条目根指针里的出生 txg。
    返回 dict：各写法下界红的 [(b, j)]、holes_plain / holes_lt（① 两种）、app_lt（出现规则）。"""
    bo = bo or {}
    po = po or {}
    birth = lambda b: bo.get(b, b[1])
    cand = sorted(candidates(h), key=lambda r: r[0])
    rs = [refs(r) | frozenset((extra or {}).get(r[0], ())) for r in cand]
    out = {k: [] for k in VARIANTS + ("holes_plain", "holes_lt", "app_lt", "app_plain")}
    seen = set(rs[0]) if rs else set()
    meta = []
    for j in range(1, len(cand)):
        R, P = cand[j], cand[j - 1]
        newly = rs[j] - rs[j - 1]
        ptr = {}
        for kind, idx in (("I", 2), ("X", 3)):
            u = R[idx]
            ptr[kind] = (u, po.get((R[0], kind), u[1]) if u is not None else None)
        def inh_pointers(strict):
            r = {}
            for kind, (u, pb) in ptr.items():
                ok = u is not None and u not in rs[j - 1] and (pb < P[0] if strict else pb <= P[0])
                r[kind] = ok
            return r
        inh_le, inh_lt = inh_pointers(False), inh_pointers(True)
        def in_inherited(b, inh):
            return any(inh[kind] and b in subtree(ptr[kind][0]) for kind in inh)
        for b in newly:
            first = b not in seen
            bb = birth(b)
            if first:
                if bb <= P[0]:
                    out["plain"].append((b, j))
                    if not inh_le["I"]:
                        out["opusI"].append((b, j))
                    if not (inh_le["I"] and inh_le["X"]):
                        out["both"].append((b, j))
                    if not in_inherited(b, inh_le):
                        out["sc_le"].append((b, j))
                    if not in_inherited(b, inh_lt):
                        out["sc_lt"].append((b, j))
            else:
                out["holes_plain"].append((b, j))
                if not in_inherited(b, inh_lt):
                    out["holes_lt"].append((b, j))
            if bb <= P[0]:
                out["app_plain"].append((b, j))
                if not in_inherited(b, inh_lt):
                    out["app_lt"].append((b, j))
        seen |= rs[j]
    out["cand"] = cand
    return out

def classify_red_units(h):
    """plain 下界红的每个单元：R_first 是不是回退根；两棵树各自是不是「从候选集之外继承来的」（指针出生 < R_prev.txg 且 R_prev 不引用）。"""
    a = analyse(h)
    cand = a["cand"]
    res = {"A_whole": 0, "B_partial": 0, "C_rollback_none": 0, "D_nonrollback": 0}
    for (b, j) in a["plain"]:
        R, P = cand[j], cand[j - 1]
        inh = [R[i] is not None and R[i] not in refs(P) and R[i][1] < P[0] for i in (2, 3)]
        if R[1] != "R":
            res["D_nonrollback"] += 1
        elif all(inh):
            res["A_whole"] += 1
        elif any(inh):
            res["B_partial"] += 1
        else:
            res["C_rollback_none"] += 1
    return res

TARGETS = [("o", 0), ("o", 1), ("o", 2), ("n", 1), ("n", 2)]
ALPHABETS = {
    "HARNESS": [("W",), ("E",), ("U",), ("K",)] + [("R", s) for s in TARGETS] + [("F", x) for x in range(4, 14)],
    "API": [("W",), ("E",), ("N",), ("U",), ("K",)] + [("R", s) for s in TARGETS] + [("F", x) for x in range(4, 14)],
    "FUT": [("W",), ("E",), ("N",), ("C",), ("M", 0), ("M", 1)] + [("R", s) for s in TARGETS] + [("F", x) for x in range(4, 14)],
    "FAULT": [("W",), ("E",), ("N",), ("K",), ("X",)] + [("L", k) for k in range(3)] + [("R", s) for s in TARGETS[:3] + TARGETS[3:4]] + [("F", x) for x in range(4, 12)],
}
KEYS = VARIANTS + ("holes_plain", "holes_lt", "app_lt")

def dfs(alpha, depth, fut=False, fault=False):
    stats = {k: 0 for k in KEYS}
    stats["histories"] = 0
    cls = {"A_whole": 0, "B_partial": 0, "C_rollback_none": 0, "D_nonrollback": 0}
    ex = {}
    def rec(h, d):
        if d > 0:
            stats["histories"] += 1
            a = analyse(h)
            for k in KEYS:
                if a[k]:
                    stats[k] += 1
                    ex.setdefault(k, (h.ops, a[k][:1]))
            if a["plain"]:
                for kk, v in classify_red_units(h).items():
                    cls[kk] += v
        if d == depth:
            return
        for op in alpha:
            n = step(h, op, fut, fault)
            if n is not None:
                rec(n, d + 1)
    rec(start(), 0)
    return stats, cls, ex

def random_histories(alpha, n, length, seed, fut=False, fault=False):
    rnd = random.Random(seed)
    stats = {k: 0 for k in KEYS}
    stats["histories"] = 0
    wrapped = 0
    ex = {}
    for _ in range(n):
        h = start()
        for _ in range(length):
            for _try in range(60):
                nh = step(h, rnd.choice(alpha), fut, fault)
                if nh is not None:
                    h = nh
                    break
            else:
                break
            # 每一步都判（与 harness 每步跑 checker 同）
            a = analyse(h)
            stats["histories"] += 1
            for k in KEYS:
                if a[k]:
                    stats[k] += 1
                    ex.setdefault(k, (h.ops, a[k][:1]))
        if h.txg > RING + 3:
            wrapped += 1
    return stats, wrapped, ex

def show(title, stats, extra=None):
    print(title)
    print("  检查的状态（每个历史前缀一张镜像）:", stats["histories"])
    for k in KEYS:
        print(f"  {k:12s} 红的镜像 {stats[k]}")
    if extra is not None:
        print("  ", extra)

# ---------- 坏镜像层：合法历史上把「这次发布新写的单元」的诞生代号盖小，各写法漏不漏 ----------
def fresh_units(cand, rs, j):
    """R_j 这次发布新写的单元：第一次出现在 R_j、诞生代号 = R_j.txg（继承来的不算）。"""
    seen = set().union(*rs[:j]) if j else set()
    return [b for b in (rs[j] - seen) if b[1] == cand[j][0]]

def user_visible_of(R, fresh):
    ui = set()
    for u in (R[2], R[3]):
        if u is not None and u in fresh:
            ui |= subtree(u)
    return ui & set(fresh)

def mutation_study(histories, ks=(1, 2, 3)):
    """对每张镜像的每条非最古候选根 R_j（R_j 之前一条是 P）做四种盖小，数各写法抓到几次。
    M1 单元：新写的单元一个一个盖到 v（v 取 0..诞生代号-1 的每个值），树表条目的根指针不动；
    M2 一整条链（用户可见）：新写的 inode/extent/数据单元的头与两个树根指针一起盖小 k，固定点单元（fx）与实例表照旧；
    M3 一整条链（含固定点）：M2 再把新写的 fx 单元盖小 k。
    抓到 = 这一根上有单元在该写法下判红。"""
    tot = {}
    def add(mode, cls, flags, ceiling_ok):
        key = (mode, cls)
        row = tot.setdefault(key, {"n": 0, "hidden": 0, **{v: 0 for v in VARIANTS}})
        row["n"] += 1
        if not ceiling_ok:
            row["hidden"] += 1
        for v in VARIANTS:
            row[v] += flags[v]
    for h in histories:
        cand = sorted(candidates(h), key=lambda r: r[0])
        rs = [refs(r) for r in cand]
        for j in range(1, len(cand)):
            R, P = cand[j], cand[j - 1]
            fresh = fresh_units(cand, rs, j)
            if not fresh:
                continue
            rollback_like = R[1] == "R"
            for b in fresh:
                cls = ("rollback根_" if rollback_like else "普通根_") + ("fx" if b[0] == "fx" else "it" if b[0] == "it" else "用户可见")
                for v in range(0, b[1]):
                    a = analyse(h, bo={b: v})
                    flags = {x: int(any(u == b and jj == j for (u, jj) in a[x])) for x in VARIANTS}
                    add("M1单元", cls, flags, v <= P[0])
            ui = user_visible_of(R, fresh)
            if not ui or rollback_like:
                continue
            for k in ks:
                if R[0] - k < 0:
                    continue
                bo = {b: b[1] - k for b in ui}
                po = {(R[0], "I"): R[0] - k, (R[0], "X"): R[0] - k}
                for mode, extra in (("M2链(用户可见)", {}), ("M3链(含固定点)", {b: b[1] - k for b in fresh if b[0] in ("fx", "it")})):
                    a = analyse(h, bo={**bo, **extra}, po=po)
                    flags = {x: int(any(jj == j for (_, jj) in a[x])) for x in VARIANTS}
                    add(f"{mode} k={k}", "普通根", flags, R[0] - k <= P[0])
    return tot

def sample_histories(alpha, n, length, seed):
    rnd = random.Random(seed)
    out = []
    for _ in range(n):
        h = start()
        for _ in range(rnd.randint(4, length)):
            for _try in range(60):
                nh = step(h, rnd.choice(alpha))
                if nh is not None:
                    h = nh
                    break
        out.append(h)
    return out

def print_study(tot):
    print(f"  {'模式':16s} {'类':14s} {'镜像数':>7s} {'天花板外':>8s} " + " ".join(f"{v:>6s}" for v in VARIANTS))
    for (mode, cls), row in sorted(tot.items()):
        print(f"  {mode:16s} {cls:14s} {row['n']:7d} {row['hidden']:8d} " + " ".join(f"{row[v]:6d}" for v in VARIANTS))

if __name__ == "__main__":
    if sys.argv[1] == "study":
        alpha = ALPHABETS["API"] + [("W",)] * 4 + [a for a in ALPHABETS["API"] if a[0] in "RF"]
        hs = sample_histories(alpha, int(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4]))
        print(f"坏镜像层：{len(hs)} 段随机合法历史（API 字母表，无故障），种子 {sys.argv[4]}；「天花板外」= 盖到的数不比 R_prev.txg 小，任何下界写法都抓不到")
        print_study(mutation_study(hs))
    else:
        name, depth = sys.argv[1], int(sys.argv[2])
        fut, fault = name == "FUT", name == "FAULT"
        stats, cls, ex = dfs(ALPHABETS[name], depth, fut, fault)
        show(f"{name} 深度 {depth}", stats, "plain 红单元按 R_first 归类 " + str(cls))

def check_appearance_rule_equals_union(histories):
    """出现规则（app_lt）= ② 下界的 sc_lt ∪ ① 的 holes_lt：逐镜像比红集。合法历史上与盖小之后都比。"""
    diff = 0
    for h in histories:
        a = analyse(h)
        if set(a["app_lt"]) != set(a["sc_lt"]) | set(a["holes_lt"]):
            diff += 1
    return diff

def reappearance_study(histories):
    """坏镜像（只红 ①）：非回退根 R_j 多引用一个更早的候选根引用过、R_{j-1} 已经不引用的单元 b（旧单元被「复活」，而这条根不是回退根）。
    ② 下界只看第一次出现（R_first 是更早那条），所以 ② 的各写法都不红；① 的 holes_plain / holes_lt 与出现规则 app_lt 红。
    R_j 是回退根时（树根指针继承）则 holes_lt 不红：那就是合法的回退，分不出。"""
    row = {}
    for h in histories:
        cand = sorted(candidates(h), key=lambda r: r[0])
        rs = [refs(r) for r in cand]
        for j in range(2, len(cand)):
            R = cand[j]
            old = set().union(*rs[:j - 1]) - rs[j - 1] - rs[j]
            for b in sorted(old, key=repr)[:3]:
                a = analyse(h, extra={R[0]: {b}})
                key = "回退根" if R[1] == "R" else "普通根"
                r = row.setdefault(key, {"n": 0, **{k: 0 for k in ("sc_lt", "holes_plain", "holes_lt", "app_lt")}})
                r["n"] += 1
                for k in ("sc_lt", "holes_plain", "holes_lt", "app_lt"):
                    r[k] += int(any(u == b and jj == j for (u, jj) in a[k]))
    return row
