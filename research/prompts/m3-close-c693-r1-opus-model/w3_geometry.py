#!/usr/bin/env python3
"""攻方腿 m3-close-c693-r1 W3：一代口径下「段宽不动树高 / 代价曲线是平的」在取样点之间与在 crates 行集上还成不成立。

纯整数算术；只读两份入库产物（E97 第五次重跑、E71 第一次重跑）里的格行，不读装置源码。
两套节点几何：
  e97_header58  E97 装置的几何：头 58 + 2 × key（带 key 区间）、叶条目 = key + 8、内部条目 = key + 53（E97 产物 acct_geom 行 header=58 child_ptr=53 的口径）
  crates        crates 今天的几何：头 115 + 2 × key（D8 已定项 11）、叶条目 = key + 8 + 4（seq）、内部条目 = key + 86
树高按「叶数 = ceil(条目 / 叶扇出)，往上每层 ceil(/ 内部扇出) 到 1」算；一次发布整棵重写时的写出节点数 = 各层节点数之和。
自证：e97_header58 几何下四档 key 的叶扇出必须等于 E97 产物 cell_side_by_side 行的 leaf_fanout（542/603/740/1087），
crates 几何下 key 22 的叶扇出必须等于 477、内部 150（crates 注释的值），不等退出 3。
弄坏开关 W3_GEOMETRY_BREAK=entry_without_seq：crates 叶条目不算 seq 那 4 字节，自证必须红。
"""
import math, os, sys

E97_PRODUCT, E71_PRODUCT = sys.argv[1], sys.argv[2]
NODE = 16384
KEYS = [7, 14, 19, 22]
BREAK = os.environ.get("W3_GEOMETRY_BREAK", "")

def geometry(name, k):
    if name == "e97_header58":
        return (NODE - 58 - 2 * k) // (k + 8), (NODE - 58 - 2 * k) // (k + 53)
    header = 115 + 2 * k
    leaf_entry = k + 8 if BREAK == "entry_without_seq" else k + 12
    return (NODE - header) // leaf_entry, (NODE - header) // (k + 86)

def levels(entries, leaf, inner):
    nodes = [max(1, math.ceil(entries / leaf))]
    while nodes[-1] > 1:
        nodes.append(math.ceil(nodes[-1] / inner))
    return nodes

def kv(line):
    return dict(t.split("=", 1) for t in line.split()[1:] if "=" in t)

ok = True
fan = {}
for line in open(E97_PRODUCT):
    if " name=cell_side_by_side " in line:
        d = kv(line)
        fan[int(d["key_bytes"])] = int(d["leaf_fanout"])
for k in KEYS:
    mine = geometry("e97_header58", k)[0]
    good = fan.get(k) == mine
    ok &= good
    print(f"E7RESULT name=selftest_leaf_fanout geometry=e97_header58 key_bytes={k} product={fan.get(k)} mine={mine} equal={int(good)}")
leaf22, inner22 = geometry("crates", 22)
good = (leaf22, inner22) == (477, 150)
ok &= good
print(f"E7RESULT name=selftest_leaf_fanout geometry=crates key_bytes=22 leaf={leaf22} inner={inner22} expected=477/150 equal={int(good)}")

# 一、crates 今天的行集 3 + 6d + 1 级台阶（D5 已定项 8 的 15 行之外加一级台阶），哪些盘数上四档 key 树高不同
for g in ("crates", "e97_header58"):
    differing = []
    for d in range(1, 4097):
        rows = 3 + 6 * d + 1
        heights = [len(levels(rows, *geometry(g, k))) for k in KEYS]
        if len(set(heights)) > 1:
            differing.append(d)
    if differing:
        runs, start = [], differing[0]
        for a, b in zip(differing, differing[1:] + [None]):
            if b != a + 1:
                runs.append(f"{start}..{a}")
                if b is not None:
                    start = b
        print(f"E7RESULT name=today_rows_height_differs_across_keys geometry={g} rows=3+6d+1 devices_1_to_4096_differing={len(differing)} ranges={','.join(runs)}")
    else:
        print(f"E7RESULT name=today_rows_height_differs_across_keys geometry={g} rows=3+6d+1 devices_1_to_4096_differing=0 ranges=none")
for d in (2, 79, 80, 142, 143):
    rows = 3 + 6 * d + 1
    for g in ("crates",):
        parts = []
        for k in KEYS:
            lv = levels(rows, *geometry(g, k))
            parts.append(f"k{k}:h{len(lv)}/nodes{sum(lv)}")
        print(f"E7RESULT name=today_rows_point geometry={g} devices={d} rows={rows} {' '.join(parts)}")

# 二、E97 那张十一项表的窄读 2t + 5d + 4（t=64、d=8 时 172，对上产物）：取样网格之间的点
for t, d in ((64, 8), (256, 8), (256, 64), (128, 64), (1024, 64)):
    n = 2 * t + 5 * d + 4
    parts = [f"k{k}:h{len(levels(n, *geometry('e97_header58', k)))}" for k in KEYS]
    print(f"E7RESULT name=e97_narrow_height geometry=e97_header58 trees={t} devices={d} entries={n} {' '.join(parts)}")

# 三、E97 产物自己的一代口径节点字节：同一格四档 key 的比值（整棵重写时代价随条目宽走，不随树高走）
cells = {}
for line in open(E97_PRODUCT):
    if " name=cell_side_by_side " in line:
        d = kv(line)
        cells[(d["trees"], d["devs"], d["reading"], int(d["key_bytes"]))] = int(d["new_node_bytes"])
for (t, d, r) in sorted({(a, b, c) for a, b, c, _ in cells}):
    vals = [cells[(t, d, r, k)] for k in KEYS]
    ratio = vals[3] / vals[0]
    print(f"E7RESULT name=e97_new_node_bytes_by_key trees={t} devs={d} reading={r} " + " ".join(f"k{k}={v}" for k, v in zip(KEYS, vals)) + f" ratio_22_over_7={ratio:.3f}")

# 四、D5:145 那句的「宽读那条臂、1024 x 64」在 E71 旧口径下过没过（E71 产物 per_statistic_wide 行）
for line in open(E71_PRODUCT):
    if " name=publish_budget " in line and " arm=per_statistic_wide " in line and " trees=1024 devices=64 " in line:
        d = kv(line)
        print(f"E7RESULT name=e71_wide_arm_1024x64 width={d['width']} entry_bytes={d['entry_bytes']} old_bytes={d['old_bytes']} old_over={d['old_over']} new_bytes={d['new_bytes']} new_over={d['new_over']}")

# 五、今天的十二项表（v1 登记十项：带设备维六项、全池四项、带树维一项按头）在 34 字节、crates 几何下的一代节点
for t, d in ((1, 2), (64, 8), (1024, 64), (8192, 64)):
    narrow = 6 * d + 4 + t
    wide = t * (6 * d + 4)
    for label, n in (("today_narrow_heads", narrow), ("all_items_by_tree", wide)):
        lv = levels(n, leaf22, inner22)
        print(f"E7RESULT name=today_table_single_generation reading={label} heads_or_trees={t} devices={d} entries={n} height={len(lv)} nodes_rewritten={sum(lv)} node_bytes={sum(lv) * NODE} over_budget={int(sum(lv) * NODE > 21474836)}")

print(f"E7RESULT name=done selftest_ok={int(ok)}")
sys.exit(0 if ok else 3)
