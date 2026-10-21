#!/usr/bin/env python3
"""攻方腿 m3-close-c693-r3 T3：改后条文 ①④⑤ 在「台阶行条数」这一维上的反例（纯算术，只读两份入库产物与正推模型输出里的扇出）。
① ⑤：E97 登记四个网格点上，窄读行数 + s 级台阶行时，四档 key 宽（叶扇出取产物 cell_side_by_side 行）在 s 取多少时树高不再彼此相同（只算叶层 1 → 2 那一道门槛）。
④：crates 几何两块盘（池级 3 + 每盘 6 = 15 行）+ s 级台阶行，21 字节 key 叶 491、22 字节 key 叶 477（正推 v3_checks.out 的 crates_leaf_fanout_one_more_key_byte 行），内部 150：每次发布重写的记账节点数。
弄坏开关 T3_BREAK=fanout_from_crates：E97 的叶扇出改用 crates 的 477，自证应退 3。"""
import os, re, sys
repo = sys.argv[1]
prod = open(os.path.join(repo, "research/results/e97-entry-encoding-single-generation-2026-10-20.out"), encoding="utf-8").read()
fan = {}
for m in re.finditer(r"name=cell_side_by_side geometry=\S+ header=58 key_bytes=(\d+) entry=\d+ leaf_fanout=(\d+) trees=(\d+) devs=(\d+) reading=narrow groups=(\d+)", prod):
    k, f, t, d, g = map(int, m.groups())
    fan[k] = f
    fan.setdefault(("groups", t, d), g)
if os.environ.get("T3_BREAK") == "fanout_from_crates":
    fan = {k: (477 if isinstance(k, int) else v) for k, v in fan.items()}
widths = sorted(k for k in fan if isinstance(k, int))
print(f"name=t3_e97_leaf_fanout widths={widths} fanouts={[fan[k] for k in widths]}")
ok = [fan[k] for k in widths] == [1087, 740, 603, 542]
for key in sorted(k for k in fan if not isinstance(k, int)):
    _, t, d = key
    g = fan[key]
    differ = [s for s in range(0, 2000) if len({1 if g + s <= fan[k] else 2 for k in widths}) > 1]
    rng = f"{differ[0]}..{differ[-1]}" if differ else "none"
    print(f"name=t3_e97_narrow_heights_differ_with_step_rows trees={t} devs={d} rows_without_steps={g} step_rows_where_four_widths_differ={rng} count={len(differ)}")
def nodes(rows, leaf, internal=150):
    if rows <= leaf:
        return 1, 1
    leaves = -(-rows // leaf)
    assert leaves <= internal
    return 2, leaves + 1
worst = None
for s in range(0, 1200):
    rows = 15 + s
    h21, n21 = nodes(rows, 491); h22, n22 = nodes(rows, 477)
    if n22 != n21:
        ratio = n22 / n21
        if worst is None or ratio > worst[0]:
            worst = (ratio, s, rows, n21, n22)
        if s in (462, 463, 476, 477):
            print(f"name=t3_crates_one_more_key_byte devices=2 step_rows={s} rows={rows} nodes_key21={n21} nodes_key22={n22} extra_nodes={n22 - n21}")
print(f"name=t3_crates_one_more_key_byte_worst ratio={worst[0]:.3f} step_rows={worst[1]} rows={worst[2]} nodes_key21={worst[3]} nodes_key22={worst[4]} stated_ratio_bound=1.028")
if not ok:
    print("name=t3_selftest fanouts_differ_from_the_product"); sys.exit(3)
print("name=t3_done")
