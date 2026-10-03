#!/usr/bin/env python3
"""B6：环窗口 = (根环里的根数 + 1) × W 里「根环里的根数」两种读法。
环几何照 root_ring.rs 文件头：R = 3 区域、每区 S 槽，第 n 次发布写区域 n mod 3 的槽 (n div 3) mod S；
回收下界 = max(F, 环里最旧有效根的 txg)（mount.rs 的 reclaim_floor），释放代 ≤ 下界才回收。F 取 0（不抬）。
读法甲：根数 = 3S（系统配置字段 × 区数，一个池寿命里不变）。
读法乙：根数 = 此刻环里读得出的根条数 c。乙-起点：只在删除起点读一次；乙-逐次：每次发布重读。
删除各批：每次发布新占 W、释放 W（引理：释放 ≥ 新写，最坏取等），单位记 W。
判：删除开始时可用空间正好 = 窗口（把池写到只剩窗口），删除全程新占 − 已回收 是否越过这笔可用。
"""
import itertools
def run(S, c0, publishes, window_slots_at_start):
    L = 3 * S
    occ = {}                                    # 槽 -> txg
    def slot(t): return (t % 3, (t // 3) % S)
    for t in range(c0): occ[slot(t)] = t         # 删除开始前环里已有 c0 条根（txg 0..c0-1）
    t = c0
    frees = []                                   # (释放代, 量)
    new_total = reclaimed_total = 0; deficit_max = 0; t_vertex = 0; first_reclaim_delay = None
    deficit_trace = []
    for i in range(1, publishes + 1):
        occ[slot(t)] = t                         # 这次发布的根落槽（取完落点之后才记根、回收：allocator.rs record_root_written_by_this_process）
        floor = min(occ.values())                # 最旧有效根
        new_total += 1                           # 这次发布新占 W（单位 1）
        frees.append((t, 1))
        still = []
        for g, q in frees:
            if g <= floor:
                reclaimed_total += q
                if first_reclaim_delay is None: first_reclaim_delay = t - g
            else: still.append((g, q))
        frees = still
        d = new_total - reclaimed_total
        deficit_trace.append(d)
        if d > deficit_max: deficit_max, t_vertex = d, i
        t += 1
    return deficit_max, t_vertex, first_reclaim_delay, deficit_trace
print("E name=ring_window columns: S L c0 | vertex(W 为单位) 顶点出现在删除后第几次发布 首次回收距释放几次发布 | 窗口甲 窗口乙(起点 c0) | 甲罩得住 乙罩得住 | 乙在第几次发布越过可用")
print("E note=乙逐次（每次发布重读环）不改结果：起点那一刻写入类发布已按 (c0+1)×W 扣窗口、把可用用到只剩这么多，之后环里根数涨了，可用不会回来")
W_4GIB = 368     # 辩方第三轮 3.1：4 GiB、K = 144 的 W_ub（槽 / 盘），只用来把单位换成槽
for S in (4, 8, 16):
    L = 3 * S
    for c0 in (4, 6, L // 2, L - 2, L - 1, L):
        vertex, tv, delay, trace = run(S, c0, 4 * L, 0)
        win_a = L + 1
        win_b = c0 + 1                           # 起点读一次（c0 ≥ L 时环满，取 L）
        win_b = min(c0, L) + 1
        covers_a = vertex <= win_a
        covers_b = vertex <= win_b
        fail_at = next((i + 1 for i, d in enumerate(trace) if d > win_b), None)
        print(f"E S={S} L={L} c0={c0} | vertex={vertex} at={tv} delay={delay} | A={win_a} Bstart={win_b} | "
              f"A_covers={covers_a} Bstart_covers={covers_b} | Bstart_fails_at={fail_at}")
print("E name=slots_4GiB_K144 W=368: " + " ".join(f"S={S}:甲={(3*S+1)*W_4GIB}槽 乙满环={(3*S+1)*W_4GIB}槽 乙新池(c=4)={5*W_4GIB}槽 乙新池(c=6)={7*W_4GIB}槽" for S in (4, 8, 16)))
# 不读盘：两种读法在准入那一刻要什么
print("E name=computability 甲=系统配置字段 root_ring_slots_per_region × ROOT_RING_REGIONS（挂载时读出，池寿命不变，O(1)）；"
      "乙=RootRingOccupancy.occupants 的条数（挂载时读环填进内存，每次发布记根时更新，O(1) 读内存；但读不出的槽不在表里）")
# 乙再加一支：环里有 u 个槽读不出（介质故障）：这些槽上若是有效根仍钉着释放，乙把它们算没了
for S in (4, 8, 16):
    L = 3 * S; u = 2
    print(f"E name=unreadable S={S} L={L} unreadable_slots={u} win_乙满环读出={(L-u)+1} win_甲={L+1} gap_slots_of_W={u}")
