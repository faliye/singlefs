"""把 runs 目录里每个变体的 name=attack 行按域合计，另把 C、C6 的 name=pairs 行按集合合计。参数：runs 目录。"""
import sys, os, re, collections
runs = sys.argv[1]
names = [l.split()[0] for l in open(os.path.join(runs, "exits.txt"), encoding="utf-8")]
cols = ["states","J1","J2","J3","J4","vm2_unsatisfiable","vm2_satisfied_but_j2","full_root_mismatch",
        "legal_states","legal_j1","legal_j2","legal_j3","legal_j4","legal_vm2_satisfied_but_j2",
        "legal_j6_premise_fails_judged","legal_j6_premise_fails","legal_j6_premise_holds"]
def fields(line):
    return dict(kv.split("=",1) for kv in line.split()[1:] if "=" in kv)
print("## 表一：每个变体、每个域的合计（light 档合法历史一族）")
print("variant\tdomain\t" + "\t".join(cols))
for name in names:
    agg = collections.OrderedDict()
    for line in open(os.path.join(runs, name + ".out"), encoding="utf-8"):
        if line.startswith("E7RESULT name=attack "):
            f = fields(line)
            d = agg.setdefault(f["domain"], collections.Counter())
            for c in cols:
                d[c] += int(f[c])
    for dom, d in agg.items():
        print(name + "\t" + dom + "\t" + "\t".join(str(d[c]) for c in cols))
print()
print("## 表二：C、C6 状态对（today_j5a / today_j5b，各 N 合计）")
print("variant\tdomain\tset\tpairs\ttoday_j5a\ttoday_j5b\tideal_j5b")
for name in names:
    agg = collections.OrderedDict()
    for line in open(os.path.join(runs, name + ".out"), encoding="utf-8"):
        if line.startswith("E7RESULT name=pairs "):
            f = fields(line)
            d = agg.setdefault((f["domain"], f["set"]), collections.Counter())
            for c in ["pairs","today_j5a","today_j5b","ideal_j5b"]:
                d[c] += int(f[c])
    for (dom, s), d in agg.items():
        if s in ("wide","cut_own_legal","cut_with_legal","cut_own_same_segment","cut_with_same_segment"):
            print(f"{name}\t{dom}\t{s}\t{d['pairs']}\t{d['today_j5a']}\t{d['today_j5b']}\t{d['ideal_j5b']}")
print()
print("## 表三：每个变体的 run 判决行与作废行")
for name in names:
    for line in open(os.path.join(runs, name + ".out"), encoding="utf-8"):
        if line.startswith("E7RESULT name=verdict scope=run") or line.startswith("E7RESULT name=void"):
            print(name + "\t" + line.strip())
