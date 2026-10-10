#!/usr/bin/env python3
"""把 sweep-*.out 汇成四份表（写进给定目录）：safety.txt（判得对不对）、cost.txt（每类写与相对今天多写）、structure.txt（I-9.13 / I-9.4 / I-9.12 / 路由 / 分隔 key）、loss.txt（单点证据丢失）。
用法：tables.py <输出目录> <sweep-*.out…>。只读 name=cell 与 name=counts 行；同一 (shape, cand, borrow) 下各几何、各种子加起来。"""
import sys, os, collections

out_dir = sys.argv[1]
files = sys.argv[2:]
os.makedirs(out_dir, exist_ok=True)

def kvs(line):
    return dict(x.split("=", 1) for x in line.split()[2:] if "=" in x)

cells = collections.defaultdict(lambda: dict(n=0, scans=0, r=0, rp=0, m=0, mp=0, win=0, out=0))
counts = collections.defaultdict(lambda: collections.defaultdict(int))
hashes = collections.defaultdict(set)
for f in files:
    for line in open(f):
        if "name=cell" in line:
            v = kvs(line)
            key = (v["shape"], v["cand"], v["reader"], v["tree"])
            e = cells[key]
            e["n"] += 1
            e["scans"] += int(v["scans"])
            r, m = int(v["r_peak"]), int(v["m_peak"])
            e["r"] = max(e["r"], r); e["m"] = max(e["m"], m)
            e["rp"] += r > 0; e["mp"] += m > 0
            e["win"] += int(v["m_scans_in_window"]); e["out"] += int(v["m_scans_outside_window"])
        elif "name=counts" in line:
            v = kvs(line)
            hashes[(v["shape"], v["seed"])].add(v["logical_hash"])
            key = (v["shape"], v["cand"], v["borrow"])
            t = counts[key]
            t["histories"] += 1
            for k, x in v.items():
                if k in ("shape", "geometry", "cand", "borrow", "seed", "logical_hash"):
                    continue
                n = int(x)
                if k.endswith("_peak"):
                    t[k] = max(t[k], n)
                else:
                    t[k] += n

with open(os.path.join(out_dir, "history-hashes.txt"), "w") as fh:
    bad = [k for k, s in hashes.items() if len(s) != 1]
    fh.write(f"segments_by_shape_and_seed={len(hashes)} with_more_than_one_logical_hash={len(bad)}\n")
    per_shape = collections.Counter(k[0] for k in hashes)
    for s in sorted(per_shape):
        fh.write(f"shape={s} seeds={per_shape[s]}\n")

order_c = ["today", "ret_a", "ret_b", "persist", "persist_x", "tsz", "fsz"]
with open(os.path.join(out_dir, "safety.txt"), "w") as fh:
    for key in sorted(cells, key=lambda k: (k[0], order_c.index(k[1]), k[2], k[3])):
        shape, cand, reader, tree = key
        if "_noP2" in reader or "_noLin" in reader:
            continue
        if reader.startswith("loss_"):
            continue
        e = cells[key]
        fh.write(f"{shape} {cand} {reader} {tree} histories={e['n']} scans={e['scans']} r_peak_max={e['r']} histories_r_pos={e['rp']} m_peak_max={e['m']} histories_m_pos={e['mp']} m_scans_in_window={e['win']} m_scans_outside_window={e['out']}\n")
    # 消融：去掉 P2 / 去掉版本谱系滤，证明装置看得见这两种错
    for key in sorted(cells, key=lambda k: (k[0], order_c.index(k[1]), k[2], k[3])):
        shape, cand, reader, tree = key
        if "_noP2" in reader or "_noLin" in reader:
            e = cells[key]
            fh.write(f"ablation {shape} {cand} {reader} {tree} histories={e['n']} r_peak_max={e['r']} histories_r_pos={e['rp']} m_peak_max={e['m']} histories_m_pos={e['mp']}\n")

def g(t, k): return t.get(k, 0)
with open(os.path.join(out_dir, "cost.txt"), "w") as fh:
    for shape, cand, borrow in sorted(counts, key=lambda k: (k[0], k[2], order_c.index(k[1]))):
        t = counts[(shape, cand, borrow)]
        base = counts[(shape, "today", borrow)]
        pub = g(t, "publishes")
        side = g(t, "units_retirement") + g(t, "units_tombstone") + g(t, "units_tree_roots_of_the_side_trees")
        side0 = g(base, "units_retirement") + g(base, "units_tombstone") + g(base, "units_tree_roots_of_the_side_trees")
        roles = g(t, "units_leaf") + side
        roles0 = g(base, "units_leaf") + side0
        ev = g(t, "drops") + g(t, "absorbs") + g(t, "borrows")
        fh.write(f"{shape} {cand} {borrow} histories={t['histories']} publishes={pub} drops={g(t,'drops')} absorbs={g(t,'absorbs')} borrows={g(t,'borrows')} "
                 f"units_leaf={g(t,'units_leaf')} units_retirement={g(t,'units_retirement')} units_tombstone={g(t,'units_tombstone')} side_tree_roots={g(t,'units_tree_roots_of_the_side_trees')} "
                 f"stub_versions={g(t,'stub_versions_written')} roles_total={roles} roles_vs_today={roles - roles0} structural_events={ev}\n")
with open(os.path.join(out_dir, "structure.txt"), "w") as fh:
    for shape, cand, borrow in sorted(counts, key=lambda k: (k[0], k[2], order_c.index(k[1]))):
        t = counts[(shape, cand, borrow)]
        ver = g(t, "versions_judged")
        fh.write(f"{shape} {cand} {borrow} versions={ver} leaf_count_sum={g(t,'leaf_count_sum')} leaf_count_peak={g(t,'leaf_count_peak')} stub_count_sum={g(t,'stub_count_sum')} "
                 f"versions_i913_false={g(t,'versions_i913_false')} versions_i94_first_false={g(t,'versions_i94_first_false')} versions_i94_second_false={g(t,'versions_i94_second_false')} versions_i94_third_false={g(t,'versions_i94_third_false')} "
                 f"versions_i912_false={g(t,'versions_i912_false')} versions_misrouted_false={g(t,'versions_misrouted_false')} versions_sep_not_derivable={g(t,'versions_with_a_separator_not_derivable_from_its_leaf')} versions_sep_matches_neither_rule={g(t,'versions_sep_matches_neither_rebuild_rule')} "
                 f"stub_sep_raised_with_room={g(t,'stub_separator_raised_with_room')} stub_sep_without_room={g(t,'stub_separator_without_room')} "
                 f"versions_in_unlink_window={g(t,'versions_in_the_unlink_window')} versions_i918_false={g(t,'versions_i918_false')}\n")
with open(os.path.join(out_dir, "loss.txt"), "w") as fh:
    for key in sorted(cells, key=lambda k: (k[0], order_c.index(k[1]), k[2], k[3])):
        shape, cand, reader, tree = key
        if reader.startswith("loss_"):
            e = cells[key]
            fh.write(f"{shape} {cand} {reader} {tree} histories={e['n']} scans={e['scans']} r_peak_max={e['r']} histories_r_pos={e['rp']} m_peak_max={e['m']} histories_m_pos={e['mp']}\n")

# ---- 前提表：每个形状（含前提族）下，今天的写路径上结构性事件与退役记录的频度（每千次主线发布），给「前两轮的数能不能并排」用
with open(os.path.join(out_dir, "premises.txt"), "w") as fh:
    for shape, cand, borrow in sorted(counts, key=lambda k: (k[0], k[2], order_c.index(k[1]))):
        if cand != "ret_a" and not (cand == "today" and shape.startswith("prem_")):
            continue
        if cand == "today" and shape.startswith("prem_") and False:
            continue
        t = counts[(shape, cand, borrow)]
        pub = g(t, "publishes")
        if pub == 0:
            continue
        per = lambda k: f"{1000 * g(t, k) / pub:.2f}"
        fh.write(f"{shape} {cand} {borrow} histories={t['histories']} publishes={pub} drops_per_1000={per('drops')} absorbs_per_1000={per('absorbs')} borrows_per_1000={per('borrows')} "
                 f"retirement_records_per_1000={per('retirement_records')} crash_points={g(t,'crash_points')} recoveries={g(t,'recoveries')} rollbacks={g(t,'rollbacks')} "
                 f"records_with_data_moved={g(t,'records_with_data_moved_to_another_leaf')} versions_in_unlink_window_share={100*g(t,'versions_in_the_unlink_window')/max(1,g(t,'versions_judged')):.1f}%\n")

# ---- 清扫族与证据在飞：崩溃点上有多少带着证据单元在飞，清扫抹了多少、漏了多少
with open(os.path.join(out_dir, "sweep.txt"), "w") as fh:
    for shape, cand, borrow in sorted(counts, key=lambda k: (k[0], order_c.index(k[1]))):
        if not shape.startswith("sweep_"):
            continue
        t = counts[(shape, cand, borrow)]
        fh.write(f"{shape} {cand} histories={t['histories']} crash_points={g(t,'crash_points')} ret_in_flight={g(t,'crash_points_with_a_retirement_unit_in_flight')} tomb_in_flight={g(t,'crash_points_with_a_tombstone_unit_in_flight')} "
                 f"zero_entry_leaf_in_flight={g(t,'crash_points_with_a_zero_entry_leaf_in_flight')} sweep_rows_deleted={g(t,'sweep_rows_deleted')} sweep_erased={g(t,'sweep_unpublished_units_erased')} sweep_evidence_missed={g(t,'sweep_unpublished_evidence_units_missed')}\n")
