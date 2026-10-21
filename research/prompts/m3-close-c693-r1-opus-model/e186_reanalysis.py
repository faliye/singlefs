#!/usr/bin/env python3
"""攻方腿 m3-close-c693-r1：拿入库的 E186 产物逐格重算胜者，换读法看判决翻不翻。

只读产物里的格行（七位向量、条款计数、W2 行数）与点行（取样点参数），不读装置源码、不读 crates/。
自证：基线读法（B1/B2 x W2 x 计/不计）算出的胜者集合计数必须与产物里同名判决行逐个相等，不等退出 3。
弄坏开关 E186_REANALYSIS_BREAK=drop_reject_plus_one：基线不加 RejectEntry 的「计」那一位，自证必须红。
变体（每个只改一处，其余照产物）：
  nodes      第 7 位从「行数」换成「记账树节点数」= ceil(行数 / 477)（crates 记账树叶容量），没有新版本的格记 0
  u5_dangling 第 6 位：NoCarry、随根留着在记账臂起作用且 h_new > 0 的格各加 1（D8 已定项 12 定案里「已定项 8 ㊀ 那类旧写者搬运的记账行」失去对象要改）
  both       两样一起
Q2 另报 w_gens 看不见时的取法：同一组其余参数、四档 w_gens 上每格取字典序最坏的向量，再取胜者。
"""
import math, os, sys
from collections import Counter, defaultdict

PRODUCT = sys.argv[1]
LEAF = 477
FLAGS = ["Ignore", "Skip", "RejectEntry", "ReadOnly", "PerBit"]
ACCT = ["Carry", "NoCarry", "KeepWithRoot"]
BREAK = os.environ.get("E186_REANALYSIS_BREAK", "")

def kv(line):
    out = {}
    for tok in line.split()[1:]:
        if "=" in tok:
            k, v = tok.split("=", 1)
            out[k] = v
    return out

points, cells, verdicts = {}, defaultdict(dict), []
with open(PRODUCT) as fh:
    for line in fh:
        if " name=cell " in line:
            d = kv(line)
            f, a = d["arm"].split("_")
            cells[int(d["point"])][(f, a)] = {k: d[k] for k in ("vector_q2_counted", "vector_q2_not_counted", "clause_units_b1", "clause_units_b2", "rows_written_w2")}
        elif " name=point_verdict " in line:
            d = kv(line)
            points[int(d["point"])] = {k: d[k] for k in ("geometry", "k_keep", "w_gens", "h_new", "registered", "g_gens", "s", "lpc", "r_tree_dim", "legal")}
        elif " name=verdict check=q1_overall_winner_accounting_arms " in line or " name=verdict check=q2_overall_winner_flags_arms " in line or " name=verdict check=q1_winners " in line:
            verdicts.append(kv(line))

def effective(f, p):
    if f == "PerBit":
        return "Skip" if p["registered"] == "4" else "ReadOnly"
    return f

def vector(p, c, f, a, b, counted, variant):
    base = [int(x) for x in c["vector_q2_counted" if counted else "vector_q2_not_counted"].split(",")]
    plus = 1 if (f == "RejectEntry" and counted and BREAK != "drop_reject_plus_one") else 0
    pos6 = int(c["clause_units_b1" if b == "b1" else "clause_units_b2"]) + plus
    rows = int(c["rows_written_w2"])
    if variant in ("u5_dangling", "both") and a in ("NoCarry", "KeepWithRoot") and effective(f, p) in ("Skip", "RejectEntry") and int(p["h_new"]) > 0:
        pos6 += 1
    pos7 = rows
    if variant in ("nodes", "both"):
        pos7 = math.ceil(rows / LEAF) if rows > 0 else 0
    return tuple(base[:5] + [pos6, pos7])

def name_set(members, order):
    return "+".join(m for m in order if m in members)

def group_of(p):
    return ("legal" if p["legal"] == "1" else "e113_style") + "_r_tree_dim_" + p["r_tree_dim"]

def analyse(b, counted, variant):
    q1_overall, q2_overall, q1_per_flag = defaultdict(Counter), defaultdict(Counter), defaultdict(Counter)
    for pi, p in points.items():
        vecs = {(f, a): vector(p, cells[pi][(f, a)], f, a, b, counted, variant) for f in FLAGS for a in ACCT}
        best = min(vecs.values())
        win = [k for k, v in vecs.items() if v == best]
        g = group_of(p)
        q1_overall[g][name_set({a for _, a in win}, ACCT)] += 1
        q2_overall[g][name_set({f for f, _ in win}, FLAGS)] += 1
        for f in FLAGS:
            bf = min(vecs[(f, a)] for a in ACCT)
            q1_per_flag[(g, f)][name_set({a for a in ACCT if vecs[(f, a)] == bf}, ACCT)] += 1
    return q1_overall, q2_overall, q1_per_flag

def render(counter):
    return "|".join(f"{k}:{v}" for k, v in sorted(counter.items()))

def selftest():
    checked, bad = 0, 0
    cache = {}
    for d in verdicts:
        reading = d["reading"]
        if "_w1_" in reading:
            continue
        b = reading.split("_")[0]
        counted = reading.endswith("_q2_counted")
        key = (b, counted)
        if key not in cache:
            cache[key] = analyse(b, counted, "registered")
        q1o, q2o, q1f = cache[key]
        g = d["group"]
        if d["check"] == "q1_overall_winner_accounting_arms":
            mine = render(q1o[g])
        elif d["check"] == "q2_overall_winner_flags_arms":
            mine = render(q2o[g])
        else:
            mine = render(q1f[(g, d["arm"])])
        checked += 1
        if mine != d["winner_sets"]:
            bad += 1
            print(f"E7RESULT name=selftest_mismatch check={d['check']} reading={reading} group={g} arm={d.get('arm')} product={d['winner_sets']} mine={mine}")
    print(f"E7RESULT name=selftest verdict_lines_checked={checked} mismatches={bad}")
    return bad == 0 and checked > 0

ok = selftest()
max_rows = max(int(c["rows_written_w2"]) for per in cells.values() for c in per.values())
print(f"E7RESULT name=max_rows_written_w2 value={max_rows} leaf_capacity={LEAF} fits_one_leaf={int(max_rows <= LEAF)}")
for variant in ("registered", "nodes", "u5_dangling", "both"):
    for b in ("b1", "b2"):
        for counted in (True, False):
            q1o, q2o, _ = analyse(b, counted, variant)
            for g in ("legal_r_tree_dim_1", "legal_r_tree_dim_2"):
                tag = f"variant={variant} reading={b}_w2_q2_{'counted' if counted else 'not_counted'} group={g}"
                print(f"E7RESULT name=q1_overall_accounting_winners {tag} winner_sets={render(q1o[g])}")
                print(f"E7RESULT name=q2_overall_flags_winners {tag} winner_sets={render(q2o[g])}")
# Q2：w_gens 看不见时，每格在四档 w_gens 上取最坏向量
for b in ("b1", "b2"):
    for counted in (True, False):
        groups = defaultdict(list)
        for pi, p in points.items():
            if p["legal"] != "1":
                continue
            key = tuple(p[k] for k in ("geometry", "k_keep", "h_new", "registered", "g_gens", "s", "lpc", "r_tree_dim"))
            groups[key].append(pi)
        tally = Counter()
        for key, pis in groups.items():
            worst = {}
            for f in FLAGS:
                for a in ACCT:
                    worst[(f, a)] = max(vector(points[pi], cells[pi][(f, a)], f, a, b, counted, "registered") for pi in pis)
            best = min(worst.values())
            tally[name_set({f for (f, a), v in worst.items() if v == best}, FLAGS)] += 1
        print(f"E7RESULT name=q2_flags_winners_w_gens_unobserved reading={b}_w2_q2_{'counted' if counted else 'not_counted'} groups={len(groups)} winner_sets={render(tally)}")
print("E7RESULT name=done")
sys.exit(0 if ok else 3)
