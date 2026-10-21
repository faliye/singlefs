#!/usr/bin/env python3
"""正推腿 m3-close-c693-r2 V1：同一批 E186 取样点上，把「第 6 位怎么数」「第 7 位怎么数」两处读法的全部组合各算一遍，
看 Carry 在被判点集（合法点里 h_new > 0，6400 点）上是不是处处在胜者里。

纯整数算术，只读两份入库产物的 cell 行与 point_verdict 行，不读装置源码，不读 crates/，不 import 别的腿的脚本。
读法两处：
  U5      vacuous  = 第一次跑产物里的第 6 位（NoCarry、随根留着下 D8 已定项 12 那半句「没写就空真」记 0）
          dangling = 第二次跑产物里的第 6 位（那半句所说的行不存在 ⇒ 要改，记 1）
  B       b1 = ㊀ 自己不计；b2 = ㊀ 自己也算一条要改的条款（产物 clause_units_b2）
  第 7 位  rows = 每代写行数（W2）；nodes = ceil(行数 / 477)（记账树叶容量 477，crates/singlefs-core/src/transaction.rs 的注释值；没有新版本的格记 0）
  Q2      counted = RejectEntry 每格另加 1（产物 vector_q2_counted 已含）；not_counted = 不加
胜者：每点 15 格（5 条 flags 臂 x 3 条记账臂）七位向量字典序最小者；记账胜者集合 = 胜者格里的记账臂。
自证（会红）：dangling x b1 x rows x counted 的胜者集合必须与第二次跑产物里 check=q1_carry_acceptance_form 的 legal_pooled 行逐点相同（Carry:6400）；
  vacuous x b1 x rows x counted 在 h_new>0 的点上必须与第一次跑产物 check=q1_overall_winner_accounting_arms 的分组计数相加后对得上（去掉 h_new=0 的 3200 点）。
  弄坏开关 V1_BREAK=swap_acct 把 NoCarry 与 KeepWithRoot 的向量对调，自证必须红（退 3）。
"""
import math, os, sys
from collections import Counter, defaultdict

FIRST, SECOND = sys.argv[1], sys.argv[2]
LEAF = 477
FLAGS = ["Ignore", "Skip", "RejectEntry", "ReadOnly", "PerBit"]
ACCT = ["Carry", "NoCarry", "KeepWithRoot"]
BREAK = os.environ.get("V1_BREAK", "")


def kv(line):
    out = {}
    for tok in line.split()[1:]:
        if "=" in tok:
            k, v = tok.split("=", 1)
            out[k] = v
    return out


def load(path):
    points, cells, verdicts = {}, defaultdict(dict), []
    with open(path) as fh:
        for line in fh:
            if " name=cell " in line:
                d = kv(line)
                f, a = d["arm"].split("_")
                if BREAK == "swap_acct" and a != "Carry":
                    a = "KeepWithRoot" if a == "NoCarry" else "NoCarry"
                cells[int(d["point"])][(f, a)] = d
            elif " name=point_verdict " in line:
                d = kv(line)
                points[int(d["point"])] = d
            elif " name=verdict " in line:
                verdicts.append(kv(line))
    return points, cells, verdicts


def vector(cell, b, counted, seventh, order="6_7"):
    base = [int(x) for x in cell["vector_q2_counted" if counted else "vector_q2_not_counted"].split(",")]
    sixth = base[5]
    if b == "b2":
        sixth += int(cell["clause_units_b2"]) - int(cell["clause_units_b1"])
    rows = int(cell["rows_written_w2"])
    seven = rows if seventh == "rows" else (math.ceil(rows / LEAF) if rows > 0 else 0)
    return tuple(base[:5] + ([sixth, seven] if order == "6_7" else [seven, sixth]))


def winners(points, cells, b, counted, seventh, order="6_7"):
    """→ {r_tree_dim: Counter(记账胜者集合 → 点数)}，只含被判点（合法且 h_new > 0）。"""
    out = defaultdict(Counter)
    for pi, p in points.items():
        if p["legal"] != "1" or int(p["h_new"]) == 0:
            continue
        vecs = {k: vector(c, b, counted, seventh, order) for k, c in cells[pi].items()}
        best = min(vecs.values())
        acct = {a for (f, a), v in vecs.items() if v == best}
        out[p["r_tree_dim"]]["+".join(a for a in ACCT if a in acct)] += 1
    return out


def render(c):
    return "|".join(f"{k}:{v}" for k, v in sorted(c.items()))


p1, c1, _ = load(FIRST)
p2, c2, v2 = load(SECOND)
ok = True

# 自证一：第二次跑 x b1 x rows x counted 全是 Carry:6400（对产物里 legal_pooled 行）
w = winners(p2, c2, "b1", True, "rows")
pooled = sum((c for c in w.values()), Counter())
product_line = [v for v in v2 if v.get("check") == "q1_carry_acceptance_form" and v.get("group") == "legal_pooled"][0]
good = render(pooled) == product_line["winner_sets"]
ok &= good
print(f"E7RESULT name=selftest_second_run_pooled mine={render(pooled)} product={product_line['winner_sets']} equal={int(good)}")

# 自证二：第一次跑 x b1 x rows x counted，h_new>0 的点，与第一次跑产物里分组行相减后对得上
_, _, v1 = load(FIRST)
product = {}
for v in v1:
    if v.get("check") == "q1_overall_winner_accounting_arms" and v.get("reading") == "b1_w2_q2_counted" and v.get("group", "").startswith("legal_r_tree_dim_"):
        product[v["group"].rsplit("_", 1)[1]] = v["winner_sets"]
w1 = winners(p1, c1, "b1", True, "rows")
for dim in ("1", "2"):
    # 产物分组含 h_new = 0 的 1600 点（三条臂定义上全等，胜者集合是 Carry+NoCarry+KeepWithRoot）；我这边只有 h_new>0，对上去掉那 1600 点的计数
    prod = Counter({kv_.split(":")[0]: int(kv_.split(":")[1]) for kv_ in product[dim].split("|")})
    prod["Carry+NoCarry+KeepWithRoot"] -= 1600
    prod = +prod
    good = w1[dim] == prod
    ok &= good
    print(f"E7RESULT name=selftest_first_run_group r_tree_dim={dim} mine={render(w1[dim])} product_minus_h_new_0={render(prod)} equal={int(good)}")

# 读法矩阵
print("E7RESULT name=matrix_header columns=u5,b,seventh,q2,r_tree_dim,winner_sets_over_judged_points,points_where_carry_is_not_a_winner")
# 位次读法：用户 2026-10-20 定第 6 位（要改几条已定条款）排在第 7 位（每代写行）之前；下面再算一遍「第 7 位排在第 6 位前」（未定的读法，只报）
for u5, (pts, cls) in (("vacuous", (p1, c1)), ("dangling", (p2, c2))):
    for seventh in ("rows", "nodes"):
        w = winners(pts, cls, "b1", True, seventh, "7_6")
        for dim in ("1", "2"):
            carry_out = sum(n for k, n in w[dim].items() if "Carry" not in k.split("+"))
            print(f"E7RESULT name=order_seventh_before_sixth u5={u5} b=b1 seventh={seventh} q2=counted r_tree_dim={dim} winners={render(w[dim])} carry_not_a_winner={carry_out}")
for u5, (pts, cls) in (("vacuous", (p1, c1)), ("dangling", (p2, c2))):
    for b in ("b1", "b2"):
        for seventh in ("rows", "nodes"):
            for counted in (True, False):
                w = winners(pts, cls, b, counted, seventh)
                for dim in ("1", "2"):
                    carry_out = sum(n for k, n in w[dim].items() if "Carry" not in k.split("+"))
                    print(f"E7RESULT name=matrix u5={u5} b={b} seventh={seventh} q2={'counted' if counted else 'not_counted'} r_tree_dim={dim} winners={render(w[dim])} carry_not_a_winner={carry_out}")
print(f"E7RESULT name=done selftest_ok={int(ok)}")
sys.exit(0 if ok else 3)
