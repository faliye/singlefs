#!/usr/bin/env python3
"""诞生代号下界第二轮辩方腿：在第一轮正推腿的抽象模型 birth_floor_v2.py（原样拷来，不改）之上加几种写法与几种坏镜像。纯 Python，不读 crates。

写法（都按「候选集按 txg 升序，相邻两条 (R⁻, R)，R 引用而 R⁻ 不引用的单元 b 要 birth(b) > R⁻.txg 或落在继承来的树里」判；
继承 = 该树根指针出生 txg < R⁻.txg 且那个落点 R⁻ 不引用，严格小于）：
  plain        原措辞，只判第一次出现（b 在更早的候选根里一条都没有）
  plain_all    原措辞，相邻对差集里每个单元都判（含再次出现）
  sc           按树分摊（第一轮「按树继承下界」不带压得住）；后缀 _all = 相邻对差集全判（β），不带 = 只判第一次出现（α 的下界半）
  scp          sc 再加「压得住」：继承来的树子树里每个单元的诞生代号都不大于树根指针的出生 txg（第二轮判决措辞原样）
  v1p          V1 的条件（inode 树根指针是继承来的）成立 ∧ 落点在压得住的继承子树下才免，否则判（= 第一轮攻方的 V3s）
  *_skip       同上，但 (R⁻, R) 之间有被丢根槽的 txg 时整对报不适用、不判（辩方给 V1 系补的一条）
坏镜像：bo = 单元 -> 改过的诞生代号；po = (根 txg, 'I'|'X') -> 改过的树根指针出生 txg。
"""
import sys
sys.dont_write_bytecode = True
import birth_floor_v2 as m
from birth_floor_v2 import candidates, refs, subtree, U

VARS = ("plain", "plain_all", "sc", "sc_all", "scp", "scp_all", "v1p", "v1p_all",
        "scp_skip", "scp_all_skip", "v1p_skip", "v1p_all_skip")

def analyse2(h, bo=None, po=None):
    bo = bo or {}
    po = po or {}
    birth = lambda b: bo.get(b, b[1])
    cand = sorted(candidates(h), key=lambda r: r[0])
    rs = [refs(r) for r in cand]
    out = {k: [] for k in VARS}
    seen = set(rs[0]) if rs else set()
    for j in range(1, len(cand)):
        R, P = cand[j], cand[j - 1]
        lost_between = any(P[0] < t < R[0] for t in h.lost)
        ptr, inh, pressed = {}, {}, {}
        for kind, idx in (("I", 2), ("X", 3)):
            u = R[idx]
            if u is None:
                inh[kind] = False
                ptr[kind] = None
                pressed[kind] = False
                continue
            pb = po.get((R[0], kind), u[1])
            ptr[kind] = u
            inh[kind] = (u not in rs[j - 1]) and pb < P[0]
            pressed[kind] = all(birth(x) <= pb for x in subtree(u))
        def exempt(b, need_pressed, need_inode):
            if need_inode and not inh["I"]:
                return False
            return any(inh[k] and (pressed[k] or not need_pressed) and b in subtree(ptr[k]) for k in ("I", "X"))
        for b in rs[j] - rs[j - 1]:
            first = b not in seen
            if birth(b) > P[0]:
                continue
            red = {"plain": first, "plain_all": True,
                   "sc": first and not exempt(b, False, False), "sc_all": not exempt(b, False, False),
                   "scp": first and not exempt(b, True, False), "scp_all": not exempt(b, True, False),
                   "v1p": first and not exempt(b, True, True), "v1p_all": not exempt(b, True, True)}
            for k, v in red.items():
                if v:
                    out[k].append((b, j))
                    if k in ("scp", "scp_all", "v1p", "v1p_all") and not lost_between:
                        out[k + "_skip" if k in ("scp", "v1p") else k + "_skip"].append((b, j))
        seen |= rs[j]
    return out

def fix_names(out):
    return out
