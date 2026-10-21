#!/usr/bin/env python3
"""E156 第 7 次重跑：k = 0 的基准族 Hh(0, —) 在各 S 下的锚点（执行员的包装，登记里的脚本没有这一族）。
用改正过 h_洞 的 anchors_e156_r7_hdong_window.py 的 run()，终点取同一 S 下各族终点的最大值（登记 W31）。
输出格式与脚本的摘要行、DUMP 行相同，族标签 Hh-none-k0。用法：python3 anchors_e156_r7_k0.py [--dump]"""
import sys
import anchors_e156_r7_hdong_window as r7

dump = '--dump' in sys.argv
for S in (4, 8, 16):
    kf, kb, families = r7.families(S)
    end = max(r7.run(S, schedule)[4] for _, schedule in families)
    rows, holes, rec, fits, end = r7.run(S, ('front', 0), end=end)
    obs = [r for r in rows if r['measured']]
    peak = max(r['q1a'] for r in obs)
    positive = sum(1 for r in obs if r['q1a'] > 0)
    print(f"S={S} family=Hh-none-k0 holes={holes} recovery_ends={rec} end={end} fits={fits} "
          f"observations={len(obs)} q1a_tt_peak={peak} q1a_tt_positive_points={positive} q1a_tt_final={obs[-1]['q1a']} "
          f"lag_peak={max(r['lag'] for r in obs)} missing_txgs_peak={max(r['h_que'] for r in obs)} live_holes_peak={max(r['h_dong'] for r in obs)} "
          f"pinned_slots_peak={max(r['pinned'] for r in obs)}")
    if dump:
        for r in obs:
            print(f"DUMP S={S} family=Hh-none-k0 txg={r['t']} floor={r['floor']} lag={r['lag']} h_que={r['h_que']} "
                  f"h_dong={r['h_dong']} pinned={r['pinned']} q1a_tree_table_lower_bound={r['q1a']}")
