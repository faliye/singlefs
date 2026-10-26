#!/usr/bin/env python3
"""汇总一个 S 的 ROW / SCAN 行成报告要引的数。用法：summary.py <该 S 的合并文件>"""
import sys
from collections import Counter, defaultdict

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import tally  # noqa: E402

CANDS = tally.CANDIDATES
DAMAGED = ("zero", "torn_half", "torn_enum")


def big(row):
    return int(row["N"]) + 1 >= 3 * int(row["S"])


def vec(row):
    return tally.vector(row)


def main():
    rows, scans = tally.load(sys.argv[1])
    rows = [r for r in rows if "N" in r]
    s = rows[0]["S"]
    print(f"== S={s} rows={len(rows)} scans={len(scans)}")
    # A：题面形
    a = [r for r in rows if r["cell"] == "A" and r["img"] in DAMAGED and big(r)]
    print(f"A 写坏镜像（N+1>=3S）{len(a)} 张：" + ", ".join(f"{c}={Counter(r[c] for r in a)}" for c in ["today", "yi_gate", "ding_yi", "ding_yi_same", "jia", "ding_jia"]))
    a_small = [r for r in rows if r["cell"] == "A" and r["img"] == "healthy" and not big(r)]
    print(f"E 第一圈健康镜像（N+1<3S）{len(a_small)} 张：yi_lit={Counter(r['yi_lit'] for r in a_small)} yi_gate={Counter(r['yi_gate'] for r in a_small)} today={Counter(r['today'] for r in a_small)}")
    # B：环洞
    b_all = [r for r in rows if r["cell"] == "B" and r["img"] in DAMAGED]
    b_land = [r for r in b_all if r["today"] == "R"]
    per_kind = Counter(r["img"] for r in b_land)
    print(f"B 写坏镜像 {len(b_all)} 张，其中落在洞上（今天判红）{len(b_land)} 张，按种类 {dict(per_kind)}")
    print("  落在洞上：" + ", ".join(f"{c}={Counter(r[c] for r in b_land)}" for c in ["today", "yi_gate", "ding_yi", "ding_yi_same", "jia", "ding_jia"]))
    b_other = [r for r in b_all if r["today"] != "R"]
    print("  没落在洞上的写坏镜像：" + ", ".join(f"{c}={Counter(r[c] for r in b_other)}" for c in ["today", "ding_yi", "jia", "ding_jia"]))
    hist_count = len({r["hist"] for r in rows if r["cell"] == "B"})
    print(f"  B 历史数（L 取值个数）{hist_count}")
    # F：钉槽
    f_rows = [r for r in rows if r["cell"] == "F"]
    by_label = defaultdict(list)
    for r in f_rows:
        by_label[r["hist"].split("/")[-1]].append(r)
    for label in ["honest", "bad_t_as_floor", "bad_p_255"]:
        group = by_label.get(label, [])
        for kind, imgs in [("健康/记录前", ("healthy", "before_record")), ("在飞/写坏", ("before_root", "zero", "torn_half", "torn_enum"))]:
            sub = [r for r in group if r["img"] in imgs]
            print(f"F {label:15s} {kind}: {len(sub)} 张：" + ", ".join(f"{c}={dict(Counter(r[c] for r in sub))}" for c in ["today", "yi_lit", "ding_yi", "jia", "ding_jia"]))
    # G
    g = [r for r in rows if r["cell"] == "G" and r["img"] in DAMAGED]
    print(f"G 写行写坏镜像 {len(g)} 张：" + ", ".join(f"{c}={dict(Counter(r[c] for r in g))}" for c in ["today", "ding_yi", "ding_yi_same", "jia", "ding_jia"]))
    # BR / Y
    for cell in ["BR", "Y"]:
        sub = [r for r in rows if r["cell"] == cell]
        print(f"{cell} {len(sub)} 张：" + ", ".join(f"{c}={dict(Counter(r[c] for r in sub))}" for c in CANDS))
    # Z2 合法镜像上各候选判红数
    for cell in ["Z2-two-holes", "Z2-hole-raised-floor", "Z2-alt-row", "Z2-alt-overwrite"]:
        sub = [r for r in rows if r["cell"] == cell and r["img"] in DAMAGED]
        print(f"{cell} 写坏镜像 {len(sub)} 张（全是合法写者）判红：" + ", ".join(f"{c}={sum(1 for r in sub if r[c] == 'R')}" for c in CANDS[:7]))
        sub_all = [r for r in rows if r["cell"] == cell]
        print(f"   该格全部 {len(sub_all)} 张里判红：" + ", ".join(f"{c}={sum(1 for r in sub_all if r[c] == 'R')}" for c in CANDS[:7]))
    # 扫描
    agg = defaultdict(lambda: [0, 0, 0, 0, 0])
    for sc in scans:
        key = (sc["cell"], sc["img"], sc["cand"])
        x = agg[key]
        x[0] += 1
        x[1] += int(sc["total"])
        x[2] += int(sc["honest"])
        x[3] += int(sc["wrong_accepted"])
        x[4] += int(sc["honest_rejected"])
    for cell in sorted({k[0] for k in agg}):
        for img in sorted({k[1] for k in agg if k[0] == cell}):
            base = agg[(cell, img, "today")]
            print(f"SCAN {cell:22s} {img:11s} 次数={base[0]} 组合={base[1]} 诚实={base[2]} | " + " ".join(f"{c}:错收{agg[(cell, img, c)][3]}/误拒{agg[(cell, img, c)][4]}" for c in CANDS[:7]))


if __name__ == "__main__":
    main()
