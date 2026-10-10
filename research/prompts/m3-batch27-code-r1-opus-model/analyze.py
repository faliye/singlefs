#!/usr/bin/env python3
"""m3-batch27-code-r1 攻方模型：读 rerun.sh 写的 summary.txt，按格出汇总行（只读、不写文件，输出到标准输出）。

用法：python3 analyze.py <summary.txt>
"""
import sys
from collections import Counter, defaultdict


def parse(path):
    rows = []
    for line in open(path, encoding="utf-8"):
        parts = line.rstrip("\n").split("\t")
        version, kind = parts[0], parts[1]
        if kind == "FIXCHECK":
            rows.append((version, kind, {"line": parts[2]}))
            continue
        fields = dict(p.split("=", 1) for p in parts[2:])
        rows.append((version, kind, fields))
    return rows


def key(f):
    return (f["b"], f["c"], f["e"], f["t"], f["l"])


def red(f):
    return set(x for x in f["red"].split(",") if x)


def main():
    rows = parse(sys.argv[1])
    j3 = {(v, key(f)): f for v, k, f in rows if k == "J3" and v in ("new", "old")}
    keys = sorted({k for (_, k) in j3})
    print(f"A0 J3 形数 new={sum(1 for v, _ in j3 if v == 'new')} old={sum(1 for v, _ in j3 if v == 'old')}")
    # A：新旧逐形比红集合
    lost = Counter()
    gained = Counter()
    lost_examples = defaultdict(list)
    gained_examples = defaultdict(list)
    for k in keys:
        n, o = j3[("new", k)], j3[("old", k)]
        for inv in red(o) - red(n):
            group = (inv, k[1], k[2], k[3], k[4])
            lost[group] += 1
            lost_examples[group].append(k[0])
        for inv in red(n) - red(o):
            group = (inv, k[1], k[2], k[3], k[4])
            gained[group] += 1
            gained_examples[group].append(k[0])
    changed = sum(1 for k in keys if red(j3[("new", k)]) != red(j3[("old", k)]))
    print(f"A1 新旧红集合不同的形 {changed} / {len(keys)}")
    for t in sorted({k[3] for k in keys}):
        c = sum(1 for k in keys if k[3] == t and red(j3[("new", k)]) != red(j3[("old", k)]))
        print(f"A1 类型段 t={t}：新旧红集合不同的形 {c}")
    print("A2 旧红而新不红（不变量 孩子故障 指针故障 类型段 层级：底数）")
    for g in sorted(lost):
        print("A2", *g, lost[g], ",".join(sorted(set(lost_examples[g]))), sep="\t")
    print("A3 新红而旧不红")
    for g in sorted(gained):
        print("A3", *g, gained[g], ",".join(sorted(set(gained_examples[g]))), sep="\t")
    # B：I-2.1 字面 V 而 checker 报成立
    for v in ("new", "old"):
        hits = [k for k in keys if j3[(v, k)]["lit21"] == "V" and j3[(v, k)]["i21"] == "H"]
        print(f"B1 {v} 字面 I-2.1 违反（这条子指针，推的读法）而 checker 报 I-2.1 成立：{len(hits)} 形")
        by = Counter((k[1], k[2], k[3]) for k in hits)
        for g in sorted(by):
            print("B2", v, *g, by[g], sep="\t")
    # C：盘序翻面：孩子故障与指针故障里点名的盘号一起 0 ↔ 1 对调（镜像形），红或部分没判集合不同的算翻面
    def swap(name):
        return name.replace("Dev0", "DevX").replace("Dev1", "Dev0").replace("DevX", "Dev1").replace("Sum0", "SumX").replace("Sum1", "Sum0").replace("SumX", "Sum1")
    for v in ("new", "old"):
        flips = []
        compared = 0
        for k in keys:
            other = (k[0], swap(k[1]), swap(k[2]), k[3], k[4])
            if other == k or other < k:
                continue
            compared += 1
            fa, fb = j3[(v, k)], j3[(v, other)]
            if (fa["red"], fa["partial"]) != (fb["red"], fb["partial"]):
                flips.append((k, other, fa["red"], fb["red"]))
        print(f"C1 {v} 盘序翻面（红或部分没判集合随故障在哪块盘而变）：比了 {compared} 对，翻面 {len(flips)} 对")
        for k, other, ra, rb in flips:
            print("C2", v, *k, "|", other[1], other[2], "|", ra, "|", rb, sep="\t")
    # D：类型段 1 / 65535 与 7 逐形比
    types = {(v, key(f)): f for v, k, f in rows if k == "J3" and v.endswith("-types")}
    same = differ = 0
    for (v, k), f in types.items():
        base_v = v.split("-")[0]
        seven = j3.get((base_v, (k[0], k[1], k[2], "7", k[4])))
        if seven is None:
            continue
        if (f["red"], f["partial"]) == (seven["red"], seven["partial"]):
            same += 1
        else:
            differ += 1
            print("D2", v, *k, f["red"], "|", seven["red"], sep="\t")
    print(f"D1 类型段 1、65535 与 7 在同一形上红与部分没判集合相同 {same} 形、不同 {differ} 形")
    # F：m3-batch26-code-r1 K2 H3 误红的第二次抽样（这份矩阵里 I-9.2 字面成立的那几形：类型段 2、孩子是那片叶容器且至少一份完好或两份都读不出、身份不动）
    ok_c = {"None", "ContentDev0", "ContentDev1", "StaleNodeDev0", "StaleNodeDev1", "ContentBoth"}
    ok_e = {"None", "FlipSum0", "FlipSum1", "FlipBoth"}
    for v in ("new", "old"):
        sample = [k for k in keys if k[3] == "2" and k[1] in ok_c and k[2] in ok_e]
        bad = [k for k in sample if "I-9.2" in red(j3[(v, k)])]
        print(f"F1 {v} I-9.2 字面成立的形 {len(sample)}，I-9.2 红 {len(bad)}")
    # G：第 27 批新写法漏判、旧写法判得出（字面 V、新 I-2.1 成立、旧 I-2.1 红）；Class2Carried 一族「内容按哪个长度算」有两读，单列
    regress = [k for k in keys if j3[("new", k)]["lit21"] == "V" and j3[("new", k)]["i21"] == "H" and j3[("old", k)]["i21"] == "V"]
    clean = [k for k in regress if k[1] != "Class2Carried"]
    print(f"G1 新漏旧判 I-2.1：{len(regress)} 形，其中不含 Class2Carried 的 {len(clean)} 形；这 {len(clean)} 形里新写法 I-4.8 成立 "
          f"{sum(1 for k in clean if j3[('new', k)]['i48'] == 'H')}、I-7.2 成立 {sum(1 for k in clean if j3[('new', k)]['i72'] == 'H')}；"
          f"类型段 {sorted({k[3] for k in clean})}")
    by = Counter((k[1], k[2]) for k in clean)
    for g in sorted(by):
        print("G2", *g, by[g], sep="\t")
    fix = {key(f): f for v, k, f in rows if k == "J3" and v == "fix"}
    if fix:
        fixed = sum(1 for k in clean if fix[k]["i21"] == "V" and fix[k]["i48"] == "V" and fix[k]["i72"] == "V")
        print(f"G3 F-α 副本上这 {len(clean)} 形 I-2.1、I-4.8、I-7.2 都红的 {fixed} 形")
        still = [k for k in keys if fix[k]["lit21"] == "V" and fix[k]["i21"] == "H"]
        print(f"G4 F-α 副本上字面 I-2.1 违反而报成立 {len(still)} 形（新写法 {sum(1 for k in keys if j3[('new', k)]['lit21'] == 'V' and j3[('new', k)]['i21'] == 'H')} 形）")
        diff_t = Counter(k[3] for k in keys if red(fix[k]) != red(j3[("new", k)]))
        print(f"G5 F-α 与新写法红集合不同的形按类型段：{dict(sorted(diff_t.items()))}")
        moved = Counter()
        for k in keys:
            for inv in red(fix[k]) ^ red(j3[("new", k)]):
                moved[(inv, "+" if inv in red(fix[k]) else "-")] += 1
        print("G6 F-α 相对新写法多红（+）少红（-）的不变量：" + ", ".join(f"{a}{b}:{n}" for (a, b), n in sorted(moved.items())))
    checks = [f["line"] for v, k, f in rows if k == "FIXCHECK"]
    if checks:
        ok_count = sum(1 for c in checks if c.endswith(" ... ok"))
        results = [c for c in checks if c.startswith("test result")]
        print(f"G7 F-α 副本上第 26、27 批那几条坏镜像：{ok_count} 条 ok；{results}")
    # E：J1
    j1 = [f for v, k, f in rows if k == "J1"]
    print(f"E1 J1 形数 {len(j1)}；转写与 CPU 实跑的门不符 {sum(1 for f in j1 if f['transcription_matches_cpu'] != 'true')}；"
          f"GPU 判了而与 CPU 不同 {sum(1 for f in j1 if f['gpu_cpu_divergence'] != 'false')}")
    for f in j1:
        print("E2", f["shape"], f["rows"], f["pages"], f["cpu_i929"], f["cpu_transcribed"], f["gpu"], sep="\t")


if __name__ == "__main__":
    main()
