#!/usr/bin/env python3
"""m3-close-c693-r3 辩方腿 T1：E186 两次跑产物上的读法矩阵，比第二轮正推的 16 格多罩 W、㊁、位次、臂集四个维度，
并把攻方派生的「按统计量」臂（零轮）与「两个统计量都有台阶行时的 NoCarry」（本腿派生，零轮）按同一套向量并进去比。
纯整数算术，只读两份入库产物的 cell 与 point_verdict 行；不读装置源码、不 import 任何别的腿的脚本。
用法：reading_matrix_wide.py <第一次跑产物> <第二次跑产物>
读法轴：
  u5       vac = 第一次跑产物里的向量（NoCarry、随根留着下 D8:383 那半句没写就空真记 0）；dang = 第二次跑产物里的向量（半句所说的行不存在 ⇒ 记 1）
  fst      ㊀ 自己（U12）计 / 不计；snd ㊁ 自己（U13）计 / 不计   （第二轮矩阵把两样绑成一个 B）
  q2       RejectEntry 另加 1 / 不加
  seventh  w2 = 记账树全部行数；w1 = 代段等于本版 txg 的行数；w2n、w1n = ceil(行数 / 477)（叶容量，没有新版本记 0）
  order    67 = 第 6 位在前（用户定）；76 = 第 7 位在前
臂集：A3 = Carry / NoCarry / 随根留着；A4 = A3 加「按统计量」；A5 = A4 加「两个统计量都有台阶行的 NoCarry」；A6 = A4 加「按统计量·按条款含义记 U5」
被判点：合法点里 h_new > 0（6400 点）。
派生臂（不是装置的一次跑）：
  PerStatistic：r_tree_dim = 1 时逐字段等于 NoCarry；r_tree_dim = 2 且记账臂起作用时 = Carry 的向量，但第 3、4 位记 0、第 7 位行数减 h_new、㊀ 自己（U12）记 1；
  NoCarryStepsBoth：r_tree_dim = 2 且记账臂起作用时 = NoCarry 的向量，但第 3、4 位记 0（第二个统计量也有台阶行，丢不了元组）。
  PerStatisticMeaning：同 PerStatistic，但 r_tree_dim = 1 且记账臂起作用时 = NoCarry 的向量去掉 U5 那一份（规则仍对「没有台阶行的统计量」有定义，D8:383 那半句不是空指称；机械谓词在这一点上记 1）。
自证（会红）：① 第二次跑每格去掉 U5 后的七位向量逐格等于第一次跑；② 每格 b2 − b1 = U12 与 U13 出现个数；
  ③ dang × (fst 不计, snd 不计) × w2 × q2 计 × 67 在 A3 上的合起来胜者集合 = 产物 legal_pooled 行；vac 同读法的 h_new>0 部分 = 第一次跑产物分组行减去 h_new = 0 的 1600 点。
  弄坏开关 MATRIX_BREAK=swap_acct 把 NoCarry 与随根留着对调，自证必须红（退 3）。
"""
import math, os, sys
from collections import Counter, defaultdict
from itertools import product

FIRST, SECOND = sys.argv[1], sys.argv[2]
LEAF = 477
FLAGS = ["Ignore", "Skip", "RejectEntry", "ReadOnly", "PerBit"]
BREAK = os.environ.get("MATRIX_BREAK", "")


def kv(line):
    return dict(tok.split("=", 1) for tok in line.split()[1:] if "=" in tok)


def load(path):
    points, cells, verdicts = {}, defaultdict(dict), []
    with open(path) as fh:
        for line in fh:
            if " name=cell " in line:
                d = kv(line)
                f, a = d["arm"].split("_")
                if BREAK == "swap_acct" and a != "Carry":
                    a = "KeepWithRoot" if a == "NoCarry" else "NoCarry"
                names = [n.split(":")[0] for n in d["clauses_false"].split("+")] if d["clauses_false"] != "none" else []
                cells[int(d["point"])][(f, a)] = dict(
                    c=[int(x) for x in d["vector_q2_counted"].split(",")],
                    n=[int(x) for x in d["vector_q2_not_counted"].split(",")],
                    b1=int(d["clause_units_b1"]), b2=int(d["clause_units_b2"]),
                    r2=int(d["rows_written_w2"]), r1=int(d["rows_written_w1"]),
                    u5=int("U5" in names), u12=int("U12" in names), u13=int("U13" in names),
                )
            elif " name=point_verdict " in line:
                points[int(kv(line)["point"])] = kv(line)
            elif " name=verdict " in line:
                verdicts.append(kv(line))
    return points, cells, verdicts


p1, c1, v1 = load(FIRST)
p2, c2, v2 = load(SECOND)
ok = True

# 自证①②
bad1 = bad2 = 0
for pi in c2:
    for key, cell in c2[pi].items():
        first = c1[pi][key]
        a = list(cell["c"]); a[5] -= cell["u5"]
        b = list(cell["n"]); b[5] -= cell["u5"]
        if a != first["c"] or b != first["n"] or cell["r2"] != first["r2"] or cell["r1"] != first["r1"]:
            bad1 += 1
        if cell["b2"] - cell["b1"] != cell["u12"] + cell["u13"]:
            bad2 += 1
cells_n = sum(len(v) for v in c2.values())
print(f"E7RESULT name=selftest_second_minus_u5_equals_first cells={cells_n} mismatches={bad1}")
print(f"E7RESULT name=selftest_b2_minus_b1_equals_u12_plus_u13 cells={cells_n} mismatches={bad2}")
ok &= bad1 == 0 and bad2 == 0


def derived(cells_of_point, point, kind):
    """→ {flags: cell}，kind = PerStatistic | NoCarryStepsBoth"""
    out = {}
    r = int(point["r_tree_dim"]); h = int(point["h_new"])
    for f in FLAGS:
        ca, nc = cells_of_point[(f, "Carry")], cells_of_point[(f, "NoCarry")]
        acts = ca["r2"] != nc["r2"]
        if kind in ("PerStatistic", "PerStatisticMeaning"):
            if not acts:
                out[f] = ca
            elif r == 1:
                if kind == "PerStatisticMeaning":
                    d = dict(nc); d["c"] = list(nc["c"]); d["n"] = list(nc["n"])
                    for k in ("c", "n"):
                        d[k][5] -= nc["u5"]
                    d["u5"] = 0
                    out[f] = d
                else:
                    out[f] = nc
            else:
                d = dict(ca)
                d["c"] = list(ca["c"]); d["n"] = list(ca["n"])
                for k in ("c", "n"):
                    d[k][2] = 0; d[k][3] = 0
                d["r2"] = ca["r2"] - h; d["r1"] = ca["r1"] - h
                d["u12"] = 1
                out[f] = d
        else:
            if not acts or r == 1:
                out[f] = nc
            else:
                d = dict(nc); d["c"] = list(nc["c"]); d["n"] = list(nc["n"])
                for k in ("c", "n"):
                    d[k][2] = 0; d[k][3] = 0
                out[f] = d
    return out


def vector(cell, fst, snd, q2, seventh, order):
    base = cell["c" if q2 else "n"]
    sixth = base[5] + (cell["u12"] if fst else 0) + (cell["u13"] if snd else 0)
    rows = cell["r2"] if seventh in ("w2", "w2n") else cell["r1"]
    seven = rows if seventh in ("w2", "w1") else (math.ceil(rows / LEAF) if rows > 0 else 0)
    return tuple(base[:5] + ([sixth, seven] if order == "67" else [seven, sixth]))


def judged(points):
    return [pi for pi, p in points.items() if p["legal"] == "1" and int(p["h_new"]) > 0]


J1, J2 = judged(p1), judged(p2)
assert J1 == J2
JUDGED = J2
PS = {"dang": {pi: derived(c2[pi], p2[pi], "PerStatistic") for pi in JUDGED},
      "vac": {pi: derived(c1[pi], p1[pi], "PerStatistic") for pi in JUDGED}}
PM = {"dang": {pi: derived(c2[pi], p2[pi], "PerStatisticMeaning") for pi in JUDGED},
      "vac": {pi: derived(c1[pi], p1[pi], "PerStatisticMeaning") for pi in JUDGED}}
SB = {"dang": {pi: derived(c2[pi], p2[pi], "NoCarryStepsBoth") for pi in JUDGED},
      "vac": {pi: derived(c1[pi], p1[pi], "NoCarryStepsBoth") for pi in JUDGED}}
ARMSETS = {"A3": ["Carry", "NoCarry", "KeepWithRoot"],
           "A4": ["Carry", "NoCarry", "KeepWithRoot", "PerStatistic"],
           "A5": ["Carry", "NoCarry", "KeepWithRoot", "PerStatistic", "NoCarryStepsBoth"],
           "A6": ["Carry", "NoCarry", "KeepWithRoot", "PerStatistic", "PerStatisticMeaning"]}


def winners(u5, fst, snd, q2, seventh, order, armset, level="overall"):
    cells, points = (c2, p2) if u5 == "dang" else (c1, p1)
    out = defaultdict(Counter)
    for pi in JUDGED:
        vecs = {}
        for a in ARMSETS[armset]:
            for f in FLAGS:
                if a == "PerStatistic":
                    cell = PS[u5][pi][f]
                elif a == "NoCarryStepsBoth":
                    cell = SB[u5][pi][f]
                elif a == "PerStatisticMeaning":
                    cell = PM[u5][pi][f]
                else:
                    cell = cells[pi][(f, a)]
                vecs[(f, a)] = vector(cell, fst, snd, q2, seventh, order)
        dim = points[pi]["r_tree_dim"]
        if level == "overall":
            best = min(vecs.values())
            acct = {a for (f, a), v in vecs.items() if v == best}
            out[dim][tuple(a for a in ARMSETS[armset] if a in acct)] += 1
        else:
            for f in FLAGS:
                sub = {a: vecs[(f, a)] for a in ARMSETS[armset]}
                best = min(sub.values())
                out[(dim, f)][tuple(a for a in ARMSETS[armset] if sub[a] == best)] += 1
    return out


def render(c):
    return "|".join(f"{'+'.join(k)}:{v}" for k, v in sorted(c.items()))


# 自证③
w = winners("dang", 0, 0, True, "w2", "67", "A3")
pooled = sum(w.values(), Counter())
line = [v for v in v2 if v.get("check") == "q1_carry_acceptance_form" and v.get("group") == "legal_pooled"][0]
mine = "|".join(f"{'+'.join(k)}:{v}" for k, v in sorted(pooled.items()))
good = mine == line["winner_sets"]
ok &= good
print(f"E7RESULT name=selftest_second_run_pooled mine={mine} product={line['winner_sets']} equal={int(good)}")
w = winners("vac", 0, 0, True, "w2", "67", "A3")
for dim in ("1", "2"):
    prod = [v for v in v1 if v.get("check") == "q1_overall_winner_accounting_arms" and v.get("reading") == "b1_w2_q2_counted" and v.get("group") == f"legal_r_tree_dim_{dim}"][0]["winner_sets"]
    pc = Counter({k.split(":")[0]: int(k.split(":")[1]) for k in prod.split("|")})
    pc["Carry+NoCarry+KeepWithRoot"] -= 1600
    pc = +pc
    mc = Counter({"+".join(k): n for k, n in w[dim].items()})
    good = mc == pc
    ok &= good
    print(f"E7RESULT name=selftest_first_run_group r_tree_dim={dim} mine={dict(sorted(mc.items()))} product_minus_h_new_0={dict(sorted(pc.items()))} equal={int(good)}")

# 矩阵
AXES = list(product(("vac", "dang"), (0, 1), (0, 1), (True, False), ("w2", "w1", "w2n", "w1n"), ("67", "76")))
summary = defaultdict(list)
for armset in ("A3", "A4", "A5", "A6"):
    for (u5, fst, snd, q2, seventh, order) in AXES:
        w = winners(u5, fst, snd, q2, seventh, order, armset)
        tag = f"armset={armset} u5={u5} fst={'count' if fst else 'skip'} snd={'count' if snd else 'skip'} q2={'counted' if q2 else 'not_counted'} seventh={seventh} order={order}"
        always = None
        for dim in ("1", "2"):
            for k in w[dim]:
                always = set(k) if always is None else always & set(k)
        print(f"E7RESULT name=matrix {tag} r1={render(w['1'])} r2={render(w['2'])} in_every_judged_point={'+'.join(sorted(always)) or 'none'}")
        summary[(armset, '+'.join(sorted(always)) or 'none')].append((u5, fst, snd, q2, seventh, order))
for (armset, always), lst in sorted(summary.items()):
    print(f"E7RESULT name=summary armset={armset} in_every_judged_point={always} readings={len(lst)}")

# 逐 flags 臂那一层（第一次跑登记第六节 accounting_winners_q1 的另一层）：只报几组关键读法
for (u5, fst, snd, q2, seventh, order) in (("vac", 0, 0, True, "w2", "67"), ("dang", 0, 0, True, "w2", "67"), ("vac", 1, 0, True, "w2", "67"), ("vac", 0, 0, True, "w2n", "67")):
    w = winners(u5, fst, snd, q2, seventh, order, "A3", level="per_flags")
    for dim in ("1", "2"):
        for f in FLAGS:
            print(f"E7RESULT name=per_flags_level u5={u5} fst={fst} snd={snd} q2={q2} seventh={seventh} order={order} r_tree_dim={dim} flags={f} winners={render(w[(dim, f)])}")

# 取样变量里哪些分得开 r_tree_dim = 1 的 NoCarry 与 NoCarry+随根留着（读法 vac, B1, w2, 计, 67）
keyvars = ["geometry", "g_gens", "s", "lpc", "w_gens", "h_new", "registered", "k_keep"]
cells, points = c1, p1
split = defaultdict(lambda: defaultdict(Counter))
for pi in JUDGED:
    if points[pi]["r_tree_dim"] != "1":
        continue
    vecs = {(f, a): vector(cells[pi][(f, a)], 0, 0, True, "w2", "67") for f in FLAGS for a in ARMSETS["A3"]}
    best = min(vecs.values())
    key = "+".join(sorted({a for (f, a), v in vecs.items() if v == best}))
    for kvar in keyvars:
        split[kvar][points[pi][kvar]][key] += 1
for kvar in keyvars:
    print(f"E7RESULT name=split_r_tree_dim_1 reading=vac_b1_w2_counted_67 variable={kvar} " + " ".join(f"{val}->{render({(k,): n for k, n in c.items()})}" for val, c in sorted(split[kvar].items())))
print(f"E7RESULT name=done selftest_ok={int(ok)}")
sys.exit(0 if ok else 3)
