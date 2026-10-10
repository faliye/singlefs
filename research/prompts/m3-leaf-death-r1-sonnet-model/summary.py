#!/usr/bin/env python3
"""把 rerun.sh 跑出来的 16 份产物压成一张稳定的摘要（排序固定、没有耗时字段、不依赖仓路径）。
用法：summary.py <runs 目录>        # 打到标准输出
      summary.py --compare <runs 目录> <期望文件>   # 逐行比，一致退 0，不同退 1 并列出前 20 处不同
runs 目录里的文件名：<候选>_<合并政策>.out。"""
import collections
import difflib
import os
import sys

RUNS = [
    ("baseline", "none"), ("baseline", "absorb"), ("baseline", "absorb_borrow"),
    ("retire", "none"), ("retire", "absorb"), ("retire", "absorb_borrow"),
    ("zero", "none"), ("zero", "absorb"), ("zero", "absorb_borrow"),
    ("all_suppressed", "none"), ("all_suppressed", "absorb"), ("all_suppressed", "absorb_borrow"),
    ("shrink_range", "none"), ("retire_shrink", "none"), ("retire_shrink", "absorb_borrow"),
    ("tight_literal", "none"),
]
TREES = ["inode", "deleted_inodes", "allocation_records_a", "central_mapping", "extent"]


def fields(line):
    return dict(part.split("=", 1) for part in line.split() if "=" in part)


def read_run(directory, candidate, merge):
    path = os.path.join(directory, f"{candidate}_{merge}.out")
    with open(path, encoding="utf-8") as handle:
        return handle.read().splitlines()


def trajectory_peaks(lines):
    best = collections.defaultdict(lambda: [0, 0])
    for line in lines:
        if not line.startswith("E7RESULT name=trajectory") or "reader=R-S" not in line:
            continue
        kv = fields(line)
        if kv["tree"] not in TREES:
            continue
        key = (kv["family"], kv["geometry"], kv["arm"], kv["tree"])
        best[key][0] = max(best[key][0], int(kv["r_peak"]))
        best[key][1] = max(best[key][1], int(kv["m_peak"]))
    return best


def label_of(line):
    """simulation / candidate_cost 行里 name=… 之后、geometry= 之前的那一段（历史的名字）。"""
    head = line.split(" geometry=", 1)[0]
    return head.split(" ", 2)[2]


def cost_totals(lines):
    seen = {}
    publishes = {}
    for line in lines:
        if line.startswith("E7RESULT name=simulation ") and " geometry=wide " in line and "write_path=N_P" in line:
            publishes[label_of(line)] = int(fields(line)["publishes"])
        if line.startswith("E7RESULT name=candidate_cost") and " geometry=wide " in line:
            kv = fields(line)
            key = label_of(line)
            value = tuple(int(kv[name]) for name in ("drops", "absorbs", "borrows", "publishes_with_ceased_identity", "range_tombstones"))
            if key in seen and seen[key] != value:
                return None
            seen[key] = value
    totals = collections.defaultdict(lambda: [0, 0, 0, 0, 0, 0])
    for key, value in seen.items():
        family = key.split("family=", 1)[1].split()[0]
        row = totals[family]
        for index, item in enumerate(value):
            row[index] += item
        row[5] += publishes.get(key, 0)
    return totals


SHORT_CANDIDATE = {"baseline": "base", "retire": "retire", "zero": "zero", "all_suppressed": "allsup", "shrink_range": "shrink", "retire_shrink": "ret+shr", "tight_literal": "tight-lit"}
SHORT_MERGE = {"none": "-", "absorb": "abs", "absorb_borrow": "abs+b"}


def short_name(candidate, merge):
    return f"{SHORT_CANDIDATE[candidate]}/{SHORT_MERGE[merge]}"


def render(directory):
    out = []
    peaks = {}
    for candidate, merge in RUNS:
        peaks[(candidate, merge)] = trajectory_peaks(read_run(directory, candidate, merge))
    header = "臂 树".ljust(28) + "".join(short_name(c, m).ljust(15) for c, m in RUNS)
    for family in ("F1", "F3", "F8"):
        for geometry in ("ring", "wide", "tight"):
            out.append(f"## 族 {family} 几何 {geometry}：R-S 读法，该族各段 r 峰值的最大值（m 非 0 时加 /m）")
            out.append(header)
            for arm in ("N", "B"):
                for tree in TREES:
                    cells = []
                    for run in RUNS:
                        value = peaks[run].get((family, geometry, arm, tree))
                        cells.append(("-" if value is None else f"{value[0]}" + ("" if value[1] == 0 else f"/m{value[1]}")).ljust(15))
                    out.append(f"{arm} {tree}".ljust(28) + "".join(cells))
            out.append("")
    out.append("## 事件数（只列 E177_CANDIDATE 或 E177_MERGE 非默认的运行；按族合计，各段取 wide 几何那一份；publishes = N_P 写路径的发布数）")
    out.append("run".ljust(32) + "family  drops  absorbs  borrows  publishes_with_ceased_identity  range_tombstones  publishes")
    for candidate, merge in RUNS:
        if candidate == "baseline" and merge == "none":
            continue
        totals = cost_totals(read_run(directory, candidate, merge))
        if totals is None:
            out.append(f"{candidate}_{merge}: 同一段两条写路径的事件数不同")
            continue
        for family in sorted(totals):
            row = totals[family]
            out.append(f"{candidate}_{merge}".ljust(32) + f"{family:<6}  {row[0]:<5}  {row[1]:<7}  {row[2]:<7}  {row[3]:<31}  {row[4]:<16}  {row[5]}")
    out.append("")
    out.append("## Q8（G-紧）R-S 判定行：基线（按 活槽+这次换下的 定盘）与 tight_literal（按活槽定盘，登记 5.1 字面读法）；格 = 状态 r峰值/m峰值 judged/excluded 段数")
    for name in ("baseline", "tight_literal"):
        lines = read_run(directory, name, "none")
        rows = []
        for line in lines:
            if line.startswith("E7RESULT name=verdict question=Q8_Q") and "reader=R-S" in line:
                kv = fields(line)
                if kv["tree"] in ("inode", "deleted_inodes", "allocation_records_a", "central_mapping", "extent", "accounting"):
                    rows.append(f"{kv['question']} {kv['arm']} {kv['tree']}: {kv['status']} r{kv['r_peak']}/m{kv['m_peak']} judged={kv['segments_judged']} excluded={kv['segments_excluded']}")
        out.append(f"### {name}")
        out.extend(sorted(rows))
    out.append("")
    out.append("## 作废标志与红的断言（装置自己的 void 判定；零候选与 shrink_range 会让「阳性对照」或「死亡记录恒 0」类断言红，原因见报告）")
    for candidate, merge in RUNS:
        lines = read_run(directory, candidate, merge)
        voiding = [line for line in lines if line.startswith("E7RESULT name=verdict scope=voiding")]
        red = sorted({fields(line)["assertion"] for line in lines if line.startswith("E7RESULT name=assertion_total") and not line.endswith("violated=0")})
        flag = fields(voiding[0])["void"] if voiding else "?"
        out.append(f"{candidate}_{merge}".ljust(32) + f"void={flag} 红的断言={','.join(red) if red else '无'}")
    return out


def main():
    if len(sys.argv) >= 2 and sys.argv[1] == "--compare":
        actual = render(sys.argv[2])
        with open(sys.argv[3], encoding="utf-8") as handle:
            expected = handle.read().splitlines()
        if actual == expected:
            print(f"摘要与期望逐行相同（{len(actual)} 行）")
            return 0
        diff = list(difflib.unified_diff(expected, actual, "期望", "这次", lineterm="", n=0))
        print("摘要与期望不同，前 20 处：")
        print("\n".join(diff[:20]))
        return 1
    print("\n".join(render(sys.argv[1])))
    return 0


if __name__ == "__main__":
    sys.exit(main())
