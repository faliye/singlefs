#!/usr/bin/env python3
"""正推腿 m3-close-c693-r2 V3：D5 已定项 4 依据两句、已定项 1 / 5 的「树高恒为 2」与「代价曲线是平的」几句，改后条文里每个数各由哪里来。

纯整数算术。只读两份入库产物（E97 第五次重跑、E71 第一次重跑），不读装置源码，不读 crates/ 源码，不 import 别的腿的脚本。
节点几何：
  e97_header58  E97 装置的几何：头 58 + 2 x key（带 key 区间），叶条目 = key + 8，内部条目 = key + 53（E97 产物 acct_geom 行 header=58 child_ptr=53 的口径）
  crates        crates 今天：头 115 + 2 x key，叶条目 = key + 8 + 4（seq），内部条目 = key + 86（叶 477、内部 150 是 transaction.rs node_capacity_of_the_node_format 的注释值）
自证：e97_header58 下四档 key 的叶扇出必须等于 E97 产物 cell_side_by_side 行的 leaf_fanout；crates 下 key 22 必须得 477 / 150；
  E97 产物 cell_side_by_side 行的 new_leaves 必须等于本脚本按条目数 groups 算的叶数（32 行逐行）。不等退 3。
  弄坏开关 V3_BREAK=leaf_without_key_range：e97_header58 叶容量不扣 2 x key（头只扣 58），自证必须红。
"""
import math, os, sys
from collections import defaultdict

E97, E71 = sys.argv[1], sys.argv[2]
NODE, KEYS = 16384, [7, 14, 19, 22]
BREAK = os.environ.get("V3_BREAK", "")
BUDGET = 21474836


def kv(line):
    return dict(t.split("=", 1) for t in line.split()[1:] if "=" in t)


def geometry(name, k):
    if name == "e97_header58":
        head = 58 if BREAK == "leaf_without_key_range" else 58 + 2 * k
        return (NODE - head) // (k + 8), (NODE - 58 - 2 * k) // (k + 53)
    return (NODE - 115 - 2 * k) // (k + 12), (NODE - 115 - 2 * k) // (k + 86)


def levels(entries, leaf, inner):
    lv = [max(1, math.ceil(entries / leaf))]
    while lv[-1] > 1:
        lv.append(math.ceil(lv[-1] / inner))
    return lv


ok = True
cells = {}
for line in open(E97):
    if " name=cell_side_by_side " in line:
        d = kv(line)
        cells[(int(d["trees"]), int(d["devs"]), d["reading"], int(d["key_bytes"]))] = d
fan = {k[3]: int(v["leaf_fanout"]) for k, v in cells.items()}
for k in KEYS:
    good = fan[k] == geometry("e97_header58", k)[0]
    ok &= good
    print(f"E7RESULT name=selftest_leaf_fanout geometry=e97_header58 key_bytes={k} product={fan[k]} mine={geometry('e97_header58', k)[0]} equal={int(good)}")
good = geometry("crates", 22) == (477, 150)
ok &= good
print(f"E7RESULT name=selftest_leaf_fanout geometry=crates key_bytes=22 mine={geometry('crates', 22)} expected=(477, 150) equal={int(good)}")
bad = 0
for (t, dv, reading, k), d in cells.items():
    mine = levels(int(d["groups"]), *geometry("e97_header58", k))[0]
    if mine != int(d["new_leaves"]):
        bad += 1
ok &= bad == 0
print(f"E7RESULT name=selftest_new_leaves rows={len(cells)} mismatches={bad}")

# 一、E97 登记的 4 个网格点上，四档 key 宽的树高（两种读法）与新口径节点字节
for t, dv in ((64, 8), (1024, 64), (8192, 8), (8192, 64)):
    for reading in ("narrow", "wide"):
        groups = int(cells[(t, dv, reading, 22)]["groups"])
        heights = [len(levels(groups, *geometry("e97_header58", k))) for k in KEYS]
        bytes_ = [int(cells[(t, dv, reading, k)]["new_node_bytes"]) for k in KEYS]
        print(f"E7RESULT name=e97_grid_point trees={t} devs={dv} reading={reading} groups={groups} heights_k7_k14_k19_k22={','.join(map(str, heights))} same_height={int(len(set(heights)) == 1)} new_node_bytes_k7_k14_k19_k22={','.join(map(str, bytes_))} ratio_22_over_7={bytes_[3] / bytes_[0]:.3f} k22_permille_of_budget={bytes_[3] * 1000 // BUDGET}")

# 二、格点之间：t = 1..8192、d = 1..64（窄读 2t+5d+4，宽读 t(5d+6)，在 64x8 / 1024x64 / 8192x8 / 8192x64 四点上与产物 groups 逐个相等）
narrow = lambda t, dv: 2 * t + 5 * dv + 4
wide = lambda t, dv: t * (5 * dv + 6)
for t, dv in ((64, 8), (1024, 64), (8192, 8), (8192, 64)):
    good = narrow(t, dv) == int(cells[(t, dv, "narrow", 22)]["groups"]) and wide(t, dv) == int(cells[(t, dv, "wide", 22)]["groups"])
    ok &= good
    print(f"E7RESULT name=selftest_group_formula trees={t} devs={dv} equal={int(good)}")
total = differ_n = differ_w = 0
for t in range(1, 8193):
    for dv in range(1, 65):
        total += 1
        differ_n += len({len(levels(narrow(t, dv), *geometry("e97_header58", k))) for k in KEYS}) > 1
        differ_w += len({len(levels(wide(t, dv), *geometry("e97_header58", k))) for k in KEYS}) > 1
print(f"E7RESULT name=e97_height_differs_between_grid_points points={total} narrow_differing={differ_n} wide_differing={differ_w}")

# 三、crates 今天的行集 3 + 6d + 一级台阶
differing = [dv for dv in range(1, 4097) if len({len(levels(3 + 6 * dv + 1, *geometry("crates", k))) for k in KEYS}) > 1]
print(f"E7RESULT name=crates_rows_height_differs_between_key_widths devices_1_to_4096_differing={len(differing)} first={differing[0]} last={differing[-1]}")
for dv in (2, 78, 79, 80):
    rows_no_step, rows_one_step = 3 + 6 * dv, 3 + 6 * dv + 1
    print(f"E7RESULT name=crates_split_point devices={dv} rows_without_step={rows_no_step} height_without_step={len(levels(rows_no_step, *geometry('crates', 22)))} rows_with_one_step={rows_one_step} height_with_one_step={len(levels(rows_one_step, *geometry('crates', 22)))}")

f21, f22 = geometry("crates", 21)[0], geometry("crates", 22)[0]
print(f"E7RESULT name=crates_leaf_fanout_one_more_key_byte key21={f21} key22={f22} fewer_entries_per_leaf={f21 - f22} permille_fewer={(f21 - f22) * 1000 // f21}")

# 四、E71：15 格里宽读 / 窄读、上界 / 窄读的条目数比，过 1% T_dirty 门槛的格
e71 = defaultdict(dict)
for line in open(E71):
    if " name=publish_budget " in line:
        d = kv(line)
        e71[(d["width"], int(d["trees"]), int(d["devices"]))][d["arm"]] = d
mid = {(t, dv): v for (w, t, dv), v in e71.items() if w == "middle"}
w_over_n = [int(v["per_statistic_wide"]["entries_per_generation"]) / int(v["per_statistic_narrow"]["entries_per_generation"]) for v in mid.values()]
u_over_n = [int(v["upper_bound"]["entries_per_generation"]) / int(v["per_statistic_narrow"]["entries_per_generation"]) for v in mid.values()]
print(f"E7RESULT name=e71_ratio_range width=middle cells={len(mid)} wide_over_narrow_min={min(w_over_n):.2f} max={max(w_over_n):.2f} upper_over_narrow_min={min(u_over_n):.2f} max={max(u_over_n):.2f}")
v = mid[(64, 8)]
print(f"E7RESULT name=e71_cell trees=64 devices=8 wide_over_narrow={int(v['per_statistic_wide']['entries_per_generation']) / int(v['per_statistic_narrow']['entries_per_generation']):.2f} upper_over_narrow={int(v['upper_bound']['entries_per_generation']) / int(v['per_statistic_narrow']['entries_per_generation']):.2f}")
for width in ("narrow", "middle", "wide", "current_implementation"):
    row = e71[(width, 1024, 64)]
    print("E7RESULT name=e71_1024x64 width=%s entry_bytes=%s " % (width, row["upper_bound"]["entry_bytes"]) + " ".join(f"{arm}:old={row[arm]['old_bytes']}/{row[arm]['old_over']},new={row[arm]['new_bytes']}/{row[arm]['new_over']}" for arm in ("upper_bound", "per_statistic_wide", "per_statistic_narrow")))
for arm in ("upper_bound", "per_statistic_wide", "per_statistic_narrow"):
    for group, widths in (("registered_widths", ("narrow", "middle", "wide")), ("current_implementation_width", ("current_implementation",))):
        cells_ = [(w, t, dv) for (w, t, dv), vv in e71.items() if w in widths]
        old = sorted(f"{w}:{t}:{dv}" for (w, t, dv) in cells_ if e71[(w, t, dv)][arm]["old_over"] == "1")
        new = sorted(f"{w}:{t}:{dv}" for (w, t, dv) in cells_ if e71[(w, t, dv)][arm]["new_over"] == "1")
        print(f"E7RESULT name=e71_over_budget arm={arm} widths={group} cells={len(cells_)} old_over={len(old)} new_over={len(new)} new_over_cells={','.join(new) or 'none'}")
print(f"E7RESULT name=done selftest_ok={int(ok)}")
sys.exit(0 if ok else 3)
