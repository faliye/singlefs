#!/usr/bin/env python3
"""C1/C2 辩方模型（纯 Python，不编 Rust、不跑装置）。

读冻结副本里 e165 装置的源码，抽出：判据表（变体名、标签）、Criterion::family、is_judged_under 与五个 judges_* / splits / compares 谓词、
新单测里写死的几个数组；再读 replay.sh 里起 E165 装置的正式跑（档、表、族）；最后拿 2026-10-04 入库产物里的 name=control 标签对一遍。
然后做两件事：
  1. C1：写死的五份集合 = 登记给这一档的集合 = 代码谓词算出的集合；每个正式跑的对照集合是不是五份集合按族过滤的子集。
  2. C2：对 is_judged_under 里「某一判据换成另一个谓词」的改坏（34 x 7 个）与 family() 里「某一判据换族」的改坏（34 x 2 个）逐个看：
     会不会改变某个正式跑的对照集合（= 真缺口），旧单测（并集写法）、新单测（五档各自核）、拿掉一档的新单测抓不抓得到。
用法：model.py <冻结副本根> <仓根>
"""
import re, sys, itertools, os

TREE = sys.argv[1]
REPO = sys.argv[2]
SRC = os.path.join(TREE, "crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs")
REPLAY = os.path.join(REPO, "scripts/experiments/replay.sh")
RESULTS = os.path.join(REPO, "research/results")
text = open(SRC, encoding="utf-8").read()
lines = text.split("\n")

def body_of(start_pat, from_line=0):
    """从匹配 start_pat 的行起，按花括号配平取整个块（返回文本）。"""
    for i in range(from_line, len(lines)):
        if re.search(start_pat, lines[i]):
            depth = 0
            out = []
            started = False
            for j in range(i, len(lines)):
                out.append(lines[j])
                depth += lines[j].count("{") - lines[j].count("}")
                if "{" in lines[j]:
                    started = True
                if started and depth == 0:
                    return "\n".join(out), i + 1
    raise SystemExit("找不到 " + start_pat)

# ---- 判据表
enum_text, enum_line = body_of(r"^\s*enum Criterion \{")
CRITERIA = re.findall(r"^\s*(\w+) => \"([^\"]+)\",", enum_text, re.M)
LABEL = dict(CRITERIA)
NAMES = [n for n, _ in CRITERIA]
print(f"判据表 {len(NAMES)} 行（枚举起于第 {enum_line} 行）")

# ---- Criterion::family
fam_text, fam_line = body_of(r"const fn family\(self\) -> Family \{", enum_line)
FAMILY = {}
for group, fam in re.findall(r"((?:\s*(?:\|\s*)?Self::\w+\s*)+)=> Family::(\w+),", fam_text):
    for v in re.findall(r"Self::(\w+)", group):
        FAMILY[v] = fam
assert set(FAMILY) == set(NAMES), set(NAMES) ^ set(FAMILY)
print("family() 起于第", fam_line, "行：", {f: sum(1 for v in FAMILY.values() if v == f) for f in sorted(set(FAMILY.values()))})

# ---- 谓词
CSETS = ["RegisteredBeforeR2", "WithJ6QnAndJ7", "WithJ6QnAndJ7SplitByI812", "WithJ6QnAndJ8SplitByI812"]
def predicate_table(fn_name):
    t, ln = body_of(r"const fn %s\(self\)" % fn_name)
    table = {}
    for group, val in re.findall(r"((?:\s*(?:\|\s*)?CriteriaSet::\w+\s*)+)=>\s*(true|false)", t):
        for v in re.findall(r"CriteriaSet::(\w+)", group):
            table[v] = (val == "true")
    assert set(table) == set(CSETS), (fn_name, table)
    return table, ln
PRED = {}
for fn in ["judges_j6qn", "judges_j7", "judges_j8", "splits_j6qn_by_i812"]:
    PRED[fn], ln = predicate_table(fn)
    print(f"{fn} 第 {ln} 行：", [k for k, v in PRED[fn].items() if v])
t_cmp, ln = body_of(r"const fn compares_the_two_record_tables\(self\)")
assert "matches!(self.table_source, RecordTableSource::Both)" in t_cmp
print("compares_the_two_record_tables 第", ln, "行：table_source = Both")

class Prof:
    def __init__(self, cset, table):
        self.cset, self.table = cset, table
    def __repr__(self):
        return f"{self.cset}/{self.table}"
    def p(self, name):
        if name == "true": return True
        if name == "false": return False
        if name == "compares": return self.table == "Both"
        if name == "not_judges_j8": return not PRED["judges_j8"][self.cset]
        if name == "compares_and_splits": return self.table == "Both" and PRED["splits_j6qn_by_i812"][self.cset]
        return PRED[name][self.cset]

# ---- is_judged_under（从源码解析每个判据挂哪个谓词）
j_text, j_line = body_of(r"const fn is_judged_under\(self, run_profile: RunProfile\)", enum_line)
JUDGED_BY = {}
arms = re.findall(r"((?:\s*(?:\|\s*)?Self::\w+\s*)+)=>\s*(\{[^}]*\}|[^,{}]+,)", j_text)
for group, expr in arms:
    e = expr.strip().rstrip(",").strip()
    if e == "true": pred = "true"
    elif "compares_the_two_record_tables" in e: pred = "compares"
    else:
        m = re.search(r"run_profile\.(\w+)\(\)", e)
        pred = m.group(1)
    for v in re.findall(r"Self::(\w+)", group):
        JUDGED_BY[v] = pred
assert set(JUDGED_BY) == set(NAMES), set(NAMES) ^ set(JUDGED_BY)
print(f"is_judged_under 起于第 {j_line} 行；谓词分布：", {p: sum(1 for v in JUDGED_BY.values() if v == p) for p in sorted(set(JUDGED_BY.values()))})

ALL_PROFILES = [Prof(c, t) for c in CSETS for t in ("Direct", "Both")]  # scan 与 direct 在这些谓词上无差
def judged(judged_by, name, prof):
    return prof.p(judged_by[name])
def control_set(judged_by, family_of, prof, families):
    out = set()
    for n in NAMES:
        every = n in ("TableComparisonSeesADifferentTable", "TableComparisonSeesADifferentRow")
        if family_of[n] not in families and not every:
            continue
        if judged(judged_by, n, prof):
            out.add(n)
    return out

# ---- 新单测里写死的数组
test_start = [i for i, l in enumerate(lines) if "fn every_criterion_has_a_positive_control_and_every_control_is_red" in l][0]
test_text = "\n".join(lines[test_start:test_start + 200])
def arr(name):
    m = re.search(r"let %s = \[(.*?)\];" % name, test_text, re.S)
    return re.findall(r"Criterion::(\w+)", m.group(1))
A = {k: arr(k) for k in ["criteria_of_every_profile", "j6qn_and_multi_disk", "j7", "j8", "table_comparison", "split"]}
print("新单测数组大小：", {k: len(v) for k, v in A.items()}, "（单测函数起于第", test_start + 1, "行）")
base, j6, j7, j8, tbl, spl = (A[k] for k in ["criteria_of_every_profile", "j6qn_and_multi_disk", "j7", "j8", "table_comparison", "split"])
r2 = base + j6 + j7
HARD = {
    "r1": set(base),
    "r2": set(r2),
    "r2-both": set(r2 + tbl),
    "r3": set(r2 + spl),
    "r5": set(base + j6 + spl + j8),
}
PROF = {"r1": Prof("RegisteredBeforeR2", "Direct"), "r2": Prof("WithJ6QnAndJ7", "Direct"), "r2-both": Prof("WithJ6QnAndJ7", "Both"),
        "r3": Prof("WithJ6QnAndJ7SplitByI812", "Direct"), "r5": Prof("WithJ6QnAndJ8SplitByI812", "Direct")}
TEST_FAMILIES = {"r1": {"LegalHistory", "Adversarial"}, "r2": {"LegalHistory", "Adversarial", "MultiDisk"}}
for k in PROF:
    TEST_FAMILIES.setdefault(k, {"LegalHistory", "Adversarial", "MultiDisk"})

# ---- 登记（我读登记原文得出的规则，与代码无关；出处见报告）
def registered(name, key):
    prof = PROF[key]
    cset = prof.cset
    if name in ("TableComparisonSeesADifferentTable", "TableComparisonSeesADifferentRow"):
        return prof.table == "Both"                       # r2 登记 5.4 T-表、T-行：--table both
    if name in ("J7aInstanceTable", "J7aAllocationRecordTreeRoot", "J7aFilesystemIdentifier", "J7bUnmountMarkerOfARebuiltRoot", "J7cUnmountMarkerWhenNothingIsApplied"):
        return cset in ("WithJ6QnAndJ7", "WithJ6QnAndJ7SplitByI812")   # r2 5.5；r3 = r2 原样；r5 换成 J8
    if name.startswith("J8"):
        return cset == "WithJ6QnAndJ8SplitByI812"          # r5 登记 5.3、5.4
    if name == "J6QnSplitByI812":
        return cset in ("WithJ6QnAndJ7SplitByI812", "WithJ6QnAndJ8SplitByI812")  # r3 登记 5.6；r5 = r3 原样
    if name in ("J6QnFixedPoint", "MultiDiskAnyCopyIsOnDisk", "MultiDiskReturnedRoot", "MultiDiskFarthestVersion", "MultiDiskAppliedSetAndLandedVersion"):
        return cset != "RegisteredBeforeR2"                 # r2 登记 5.1、5.4；r5 登记 5.1 认多盘
    return True                                             # 原登记 5.4 的 18 条每档都判

print("\n== C1 逐档：写死的集合 vs 登记规则 vs 代码谓词（is_judged_under 解析）")
ok = True
for key in ["r1", "r2", "r2-both", "r3", "r5"]:
    reg = {n for n in NAMES if registered(n, key)}
    code = control_set(JUDGED_BY, FAMILY, PROF[key], TEST_FAMILIES[key])
    h = HARD[key]
    print(f"{key:8s} 写死 {len(h):2d}  登记规则 {len(reg):2d}  代码 {len(code):2d}  "
          f"写死=登记 {h == reg}  写死=代码 {h == code}")
    if h != reg: print("   写死-登记", sorted(h - reg), "登记-写死", sorted(reg - h)); ok = False
    if h != code: print("   写死-代码", sorted(h - code), "代码-写死", sorted(code - h)); ok = False
cover = set().union(*HARD.values())
print("五档并起来罩全表：", cover == set(NAMES), "（缺：", sorted(set(NAMES) - cover), "）")

# ---- 正式跑
rs = open(REPLAY, encoding="utf-8").read().split("\n")
RUNS = []
for i, l in enumerate(rs, 1):
    if "--bin e165_replay_bounded_enumeration" not in l: continue
    args = l.split("--bin e165_replay_bounded_enumeration --")[1].split("|")[0].split()
    def opt(name, default=None):
        return args[args.index(name) + 1] if name in args else default
    fam = opt("--family")
    if fam in ("root_choice", "root_choice_faults"): continue
    prof = opt("--profile", "r1")
    if prof == "r4": continue
    tbl_ = opt("--table", "direct")
    RUNS.append(dict(line=i, family=fam, profile=prof, table=tbl_, only=opt("--only"), disk=opt("--disk-order"), parity=opt("--marker-parity"), bounds=opt("--bounds")))
FAMNAME = {"legal": "LegalHistory", "adversarial": "Adversarial", "multi_disk": "MultiDisk"}
def key_of(run):
    if run["profile"] == "r1": return "r1"
    if run["profile"] == "r2": return "r2-both" if run["table"] == "both" else "r2"
    if run["profile"] == "r3": return "r3"
    if run["profile"] == "r5": return "r5"
print("\n== replay.sh 里起 E165 装置的正式跑（不含 root_choice / root_choice_faults 两族，它们有自己的对照）")
formal_sets = {}
for r in RUNS:
    k = key_of(r); fams = {FAMNAME[r["family"]]}
    s = control_set(JUDGED_BY, FAMILY, PROF[k], fams)
    r["key"], r["fams"], r["set"] = k, fams, s
    print(f"replay.sh 第 {r['line']} 行  --family {r['family']:10s} --profile {r['profile']} --table {r['table']:6s}"
          f" only={r['only']} disk={r['disk']} parity={r['parity']} -> 对照 {len(s)} 条，档 {k}")
    formal_sets.setdefault(k, set()).update(s)
print("\n各档正式跑对照集合的并 vs 写死的集合：")
for k in ["r1", "r2", "r2-both", "r3", "r5"]:
    u = formal_sets.get(k, set())
    sub = u <= HARD[k]
    print(f"{k:8s} 正式跑的并 {len(u):2d} 条，写死 {len(HARD[k]):2d} 条，并 ⊆ 写死：{sub}，并 = 写死：{u == HARD[k]}", "" if u == HARD[k] else f" 写死里正式跑不打印的 {len(HARD[k]-u)} 条")

# ---- 入库产物里的 name=control 标签（2026-10-04，旧数据，只作旁证）
PRODUCTS = {
    ("r1", "legal"): "r5-segment1-r1-legal-family", ("r1", "adversarial"): "r5-segment1-r1-adversarial-family",
    ("r2", "multi_disk"): "r5-segment1-r2-multi-disk",
    ("r2-both", "legal"): "r5-segment1-r2-table-both-legal-family", ("r2-both", "adversarial"): "r5-segment1-r2-table-both-adversarial-family",
    ("r2-both", "multi_disk"): "r5-segment1-r2-table-both-multi-disk",
    ("r3", "adversarial"): "r5-segment1-r3-adversarial-family",
    ("r5", "legal"): "r5-segment3-r5-legal-family", ("r5", "adversarial"): "r5-segment3-r5-adversarial-family", ("r5", "multi_disk"): "r5-segment3-r5-multi-disk",
}
LABEL2NAME = {v: k for k, v in LABEL.items()}
print("\n== 2026-10-04 入库产物的 name=control 标签 vs 代码谓词算出的这一跑的集合（旧数据，旁证）")
for (k, fam), stem in PRODUCTS.items():
    path = os.path.join(RESULTS, f"e165-replay-bounded-enumeration-2026-10-04-{stem}.out")
    if not os.path.exists(path):
        print(k, fam, "产物不在：", path); continue
    got = {LABEL2NAME[m] for m in re.findall(r"name=control criterion=(\S+)", open(path, encoding="utf-8").read())}
    want = control_set(JUDGED_BY, FAMILY, PROF[k], {FAMNAME[fam]})
    print(f"{k:8s} {fam:10s} 产物 {len(got):2d} 种标签，谓词算 {len(want):2d}，相等：{got == want}")

# ---- C2：改坏
PREDS = ["true", "false", "judges_j6qn", "judges_j7", "judges_j8", "splits_j6qn_by_i812", "compares"]
FORMAL = [(r["key"], frozenset(r["fams"])) for r in RUNS]   # 去重
FORMAL = sorted(set(FORMAL), key=lambda x: (x[0], sorted(x[1])))
def formal_sets_under(judged_by, family_of):
    return {(k, f): control_set(judged_by, family_of, PROF[k], set(f)) for k, f in FORMAL}
BASE_FORMAL = formal_sets_under(JUDGED_BY, FAMILY)

def detect_new(judged_by, family_of, keys=("r1", "r2", "r2-both", "r3", "r5")):
    for k in keys:
        if control_set(judged_by, family_of, PROF[k], TEST_FAMILIES[k]) != HARD[k]:
            return True
    return False
# 旧单测（第一轮冻结副本第 18274 行起）：r1 只核全红；r2 核「读数里没有拆行」；r3、r5、r3-both 并起来核每个判据至少一条读数、全红
R3 = Prof("WithJ6QnAndJ7SplitByI812", "Direct"); R3B = Prof("WithJ6QnAndJ7SplitByI812", "Both")
def detect_old(judged_by, family_of):
    r2s = control_set(judged_by, family_of, PROF["r2"], TEST_FAMILIES["r2"])
    if "J6QnSplitByI812" in r2s: return True
    union = set()
    for prof in (R3, PROF["r5"], R3B):
        union |= control_set(judged_by, family_of, prof, TEST_FAMILIES["r3"])
    return union != set(NAMES)
# 旧单测里没有的写法算「全红」靠运行：增加一条对照可能红也可能不红，这里按「加一条、红」算，对旧单测是宽的（旧单测少抓不会多抓）

def affects_formal(judged_by, family_of):
    cur = formal_sets_under(judged_by, family_of)
    drop = sum(1 for key in cur if BASE_FORMAL[key] - cur[key])
    add = sum(1 for key in cur if cur[key] - BASE_FORMAL[key])
    return drop, add

import collections
print("\n== C2 改坏 A：is_judged_under 里把某一判据的谓词换成另一个（%d x %d）" % (len(NAMES), len(PREDS) - 1))
shapes = collections.OrderedDict()
shapes["旧单测（并集）"] = lambda jb, fo: detect_old(jb, fo)
shapes["新单测（五档各自核）"] = lambda jb, fo: detect_new(jb, fo)
for drop_key in ["r1", "r2", "r2-both", "r3", "r5"]:
    keys = tuple(k for k in ("r1", "r2", "r2-both", "r3", "r5") if k != drop_key)
    shapes[f"新单测去掉 {drop_key} 一档"] = (lambda keys: (lambda jb, fo: detect_new(jb, fo, keys)))(keys)
stat = {s: dict(real_drop=0, real_drop_caught=0, real_add=0, real_add_caught=0, noreal=0, noreal_caught=0) for s in shapes}
total = real_drop = real_add = 0
escapes = {s: [] for s in shapes}
for n in NAMES:
    for p in PREDS:
        if p == JUDGED_BY[n]: continue
        jb = dict(JUDGED_BY); jb[n] = p
        drop, add = affects_formal(jb, FAMILY)
        # 同一个改坏在 12 个档上的效果里，若没有一个正式跑受影响，记 noreal（谓词换了但正式跑用到的档上读数没变）
        total += 1
        kind = "drop" if drop else ("add" if add else "no")
        if kind == "drop": real_drop += 1
        elif kind == "add": real_add += 1
        for s, f in shapes.items():
            caught = f(jb, FAMILY)
            st = stat[s]
            if kind == "drop":
                st["real_drop"] += 1; st["real_drop_caught"] += caught
                if not caught: escapes[s].append((n, p))
            elif kind == "add":
                st["real_add"] += 1; st["real_add_caught"] += caught
            else:
                st["noreal"] += 1; st["noreal_caught"] += caught
print(f"改坏 {total} 个：正式跑某个跑丢一条对照（真缺口）{real_drop} 个，只多一条对照 {real_add} 个，正式跑集合不变 {total-real_drop-real_add} 个")
print(f"{'单测写法':28s} {'真缺口抓到/共':>14s} {'只多一条抓到/共':>16s} {'正式跑不变却红':>14s}")
for s, st in stat.items():
    print(f"{s:28s} {st['real_drop_caught']:>6d}/{st['real_drop']:<7d} {st['real_add_caught']:>7d}/{st['real_add']:<8d} {st['noreal_caught']:>8d}/{st['noreal']}")
print("\n真缺口里各写法放过的条数与前几个例子（判据 -> 换成的谓词）：")
for s in shapes:
    e = escapes[s]
    print(f"  {s}：放过 {len(e)} 个", e[:6] if e else "")
# ---- 单格翻转（对应变异行 3383、3407、3496 那一类：只在一个档上改）
print("\n== C2 改坏 B：只在一个档的一格上翻（变异行 3383、3407、3496 的形）")
CELL_PROFILES = {"r1": PROF["r1"], "r2": PROF["r2"], "r2-both": PROF["r2-both"], "r3": PROF["r3"], "r5": PROF["r5"], "r3-both(旧单测才有)": R3B}
rows = []
for flip_key, flip_prof in CELL_PROFILES.items():
    for n in NAMES:
        if not judged(JUDGED_BY, n, flip_prof):  # 只翻「判着的」→ 不判（丢对照）
            continue
        # 正式跑受影响吗：该档上任一正式跑的集合含 n
        real = any(n in BASE_FORMAL[(k, f)] for (k, f) in FORMAL if PROF[k].cset == flip_prof.cset and PROF[k].table == flip_prof.table)
        # 新单测抓到：五档里有与翻格同 (cset, table) 的档，且那档的族集合含 n 的族（全族，所以必含）
        new_caught = any(PROF[k].cset == flip_prof.cset and PROF[k].table == flip_prof.table for k in HARD)
        # 旧单测抓到：n 在旧并集里是否只由这一档提供（r3 / r5 / r3-both）；翻 r1、r2 的格旧单测不抓
        old_caught = False
        old_profs = [R3, PROF["r5"], R3B]
        if any(flip_prof.cset == o.cset and flip_prof.table == o.table for o in old_profs):
            others = [o for o in old_profs if not (flip_prof.cset == o.cset and flip_prof.table == o.table)]
            old_caught = not any(judged(JUDGED_BY, n, o) for o in others)
        rows.append((flip_key, n, real, new_caught, old_caught))
for flip_key in CELL_PROFILES:
    rs_ = [r for r in rows if r[0] == flip_key]
    real_rs = [r for r in rs_ if r[2]]
    print(f"{flip_key:22s} 判着的格 {len(rs_):2d}，其中正式跑用到的 {len(real_rs):2d}；新单测抓到 {sum(r[3] for r in real_rs):2d}，旧单测抓到 {sum(r[4] for r in real_rs):2d}")
tot_real = [r for r in rows if r[2]]
print(f"合计 正式跑用到的格 {len(tot_real)}：新单测抓到 {sum(r[3] for r in tot_real)}，旧单测抓到 {sum(r[4] for r in tot_real)}")
print("旧单测抓到的格：", [(r[0], r[1]) for r in tot_real if r[4]])

# ---- 失去唯一读数（C2 第三问）
print("\n== C2 第三问：拿掉某一档，哪些判据失去（并集意义上的）唯一一份读数")
def sole_losses(profiles, families_of):
    res = {}
    cov = collections.defaultdict(list)
    for pk, (prof, fams) in profiles.items():
        for n in control_set(JUDGED_BY, FAMILY, prof, fams):
            cov[n].append(pk)
    return cov
OLD_PROFILES = {"r1": (PROF["r1"], TEST_FAMILIES["r1"]), "r2": (PROF["r2"], TEST_FAMILIES["r2"]), "r3": (R3, TEST_FAMILIES["r3"]), "r5": (PROF["r5"], TEST_FAMILIES["r5"]), "r3-both": (R3B, TEST_FAMILIES["r3"])}
NEW_PROFILES = {k: (PROF[k], TEST_FAMILIES[k]) for k in PROF}
for title, profiles, drops in [
        ("旧单测（r1、r2 只核全红；并集取 r3、r5、r3-both）", OLD_PROFILES, [("r3",), ("r3-both",), ("r3", "r3-both")]),
        ("新单测（五档）", NEW_PROFILES, [("r1",), ("r2",), ("r2-both",), ("r3",), ("r5",)])]:
    for d in drops:
        remaining = {k: v for k, v in profiles.items() if k not in d}
        # 旧单测里 r1、r2 不进并集
        if title.startswith("旧"):
            union_profiles = {k: v for k, v in remaining.items() if k in ("r3", "r5", "r3-both")}
        else:
            union_profiles = remaining
        cov = sole_losses(union_profiles, None)
        lost = sorted(n for n in NAMES if not cov.get(n))
        print(f"{title} 去掉 {'+'.join(d)}：并集里没有读数的判据 {len(lost)}", [LABEL[n] for n in lost])

# ---- 改坏 C：family() 换族。新单测（全族）抓不到；按正式跑的族逐个核的写法抓得到
print("\n== C2/C1 改坏 C：Criterion::family 把某一判据换到另一族（%d x 2）" % len(NAMES))
FAMS = ["LegalHistory", "Adversarial", "MultiDisk"]
def detect_faithful(judged_by, family_of):
    # 按 replay.sh 的每个正式跑各核一次它的对照集合
    cur = formal_sets_under(judged_by, family_of)
    exp = BASE_FORMAL
    return any(cur[k] != exp[k] for k in cur)
tot = real = new_c = old_c = faith_c = 0
for n in NAMES:
    for fam in FAMS:
        if fam == FAMILY[n]: continue
        fo = dict(FAMILY); fo[n] = fam
        tot += 1
        drop, add = affects_formal(JUDGED_BY, fo)
        if drop or add:
            real += 1
            new_c += detect_new(JUDGED_BY, fo); old_c += detect_old(JUDGED_BY, fo); faith_c += detect_faithful(JUDGED_BY, fo)
print(f"换族 {tot} 个，改变某个正式跑的对照集合 {real} 个；新单测抓到 {new_c}，旧单测抓到 {old_c}，按正式跑逐个核的写法抓到 {faith_c}")
print("例：", [(n, FAMILY[n], '->', fam) for n in NAMES[:0]])

# ---- 补：不影响正式跑却让新单测红的那些改坏、换族放过的例子、两种单测的开销
print("\n== 补 1：谓词换了、正式跑的对照集合一个都没变（13 个）里新单测红的那些，红在哪一档")
for n in NAMES:
    for p in PREDS:
        if p == JUDGED_BY[n]: continue
        jb = dict(JUDGED_BY); jb[n] = p
        drop, add_ = affects_formal(jb, FAMILY)
        if drop or add_: continue
        reds = [k for k in PROF if control_set(jb, FAMILY, PROF[k], TEST_FAMILIES[k]) != HARD[k]]
        print(f"  {LABEL[n]:34s} -> {p:20s} 新单测红在 {reds if reds else '不红'}")
print("\n== 补 2：换族放过新单测的例子（按判据 -> 换到的族；改变了哪个正式跑）")
shown = 0
esc = 0
for n in NAMES:
    for fam in FAMS:
        if fam == FAMILY[n]: continue
        fo = dict(FAMILY); fo[n] = fam
        cur = formal_sets_under(JUDGED_BY, fo)
        diff = [(k, sorted(f)) for (k, f) in cur if cur[(k, f)] != BASE_FORMAL[(k, f)]]
        if diff and not detect_new(JUDGED_BY, fo):
            esc += 1
            if shown < 6:
                shown += 1
                print(f"  {LABEL[n]} {FAMILY[n]} -> {fam}：正式跑变了 {diff[:3]}")
print("  放过新单测且改变正式跑的换族共", esc, "个")
print("\n== 补 3：开销（调用 positive_control_readings 的次数与每次的判据个数，不含各条对照自己的扫描量）")
old_calls = {"r1": len(control_set(JUDGED_BY, FAMILY, PROF["r1"], TEST_FAMILIES["r1"])), "r2": len(control_set(JUDGED_BY, FAMILY, PROF["r2"], TEST_FAMILIES["r2"])),
             "r3": len(control_set(JUDGED_BY, FAMILY, R3, TEST_FAMILIES["r3"])), "r5": len(control_set(JUDGED_BY, FAMILY, PROF["r5"], TEST_FAMILIES["r5"])),
             "r3-both": len(control_set(JUDGED_BY, FAMILY, R3B, TEST_FAMILIES["r3"]))}
new_calls = {k: len(HARD[k]) for k in HARD}
print("旧单测", old_calls, "共", len(old_calls), "次调用、", sum(old_calls.values()), "条判据的对照")
print("新单测", new_calls, "共", len(new_calls), "次调用、", sum(new_calls.values()), "条判据的对照")
faithful = [len(BASE_FORMAL[(k, f)]) for (k, f) in FORMAL]
print("按正式跑逐个核（每个 (档, 族) 一次）", len(faithful), "次调用、", sum(faithful), "条判据的对照")

print("\n== 补 4：四行变异里前三行（锚在 is_judged_under）在旧并集单测与新单测下的形（整组一起换，同变异表的原文替换文）")
BASE18 = set(base)
rowdefs = [
    ("3383 基础 18 条换成 !judges_j8（r5 档不判）", {n: "not_judges_j8" for n in BASE18}),
    ("3407 T_table、T_row 换成 compares && splits（r2 两份表都取不判）", {n: "compares_and_splits" for n in ("TableComparisonSeesADifferentTable", "TableComparisonSeesADifferentRow")}),
    ("3496 基础 18 条换成 judges_j6qn（r1 档不判）", {n: "judges_j6qn" for n in BASE18}),
]
for title, repl in rowdefs:
    jb = dict(JUDGED_BY); jb.update(repl)
    drop, add_ = affects_formal(jb, FAMILY)
    print(f"{title}：改变正式跑的对照集合 {drop} 个丢、{add_} 个多；旧并集单测红：{detect_old(jb, FAMILY)}；新单测红：{detect_new(jb, FAMILY)}；"
          f"新单测红在 {[k for k in PROF if control_set(jb, FAMILY, PROF[k], TEST_FAMILIES[k]) != HARD[k]]}")
sys.exit(0 if ok else 1)
