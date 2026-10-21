#!/usr/bin/env python3
# E156 第 7 次重跑登记的独立锚点（问题单 H1 的族）。不 import 装置、不读 crates/。只用下列条款句与读法：
#   根环：txg u 落 u mod 3S 那一槽（D22 已定项 1 的 R = 3、S；第 2 次登记 A2 核过一一对应）；区域设备 [0, 1, 0]，txg u 的根落盘 [0,1,0][u % 3]。
#   可再分配 = 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)（D16 已定项 1）；这些族里不抬 F ⇒ F_生效 = 0。
#   环里最旧有效根 = min(A ∪ B)：A = 环里有根的 txg；B = 这一次挂载那一刻读回全零（从没写过）、这次挂载还没写掉的槽，
#     各取「落到这一槽、小于这次挂载第一个 txg 的最小 txg」（第 6 次登记 W15–W18；D16 已定项 1「读坏的槽与读回全零的槽」一句）。
#   每一次发布（连空发布、写行、暖机、崩溃那一次）都换下上一版的树表单元（1 槽）；mkfs 之后 txg 1、2 两次暖机是零单元发布，不换。
#   崩溃 c2（记录已持久、根槽没持久）：那个 txg 在环里没有根；之后可写挂载：写行 = 洞 + 1，暖机直到两块盘上各有一条本实例的根（D16 已定项 8）。
# 第 7 次重跑执行员对 S20 的改正(只改 h_洞 这一行)：r2 R2 第二读法与第 4 次脚本(anchors_e156_r4.py 第 738 行)的 h_洞 都带窗口左端 oldest <= h，登记里的 r7 脚本漏了它。
# 单元只数树表：q1a_tt 是 Q1a（甲-T1 多扣）的下界，不是真值。
import itertools, sys

REGION_DEVICES = [0, 1, 0]
def dev(u): return REGION_DEVICES[u % 3]
GAP = 2           # 造洞之后隔几次工作负载发布才许造下一个（第 4 次登记表头第 4 行）
FRONT_FIRST = 5   # 前位置第一个洞最早的 txg

def history(S, schedule):
    """schedule：('front'|'back', 洞数) 或 ('slot', 位置, 圈数)。交回 events、holes、recovery_ends、kinds。"""
    P = 3 * S
    events = {0: 'root', 1: 'root', 2: 'root', 3: 'root'}
    kinds = {0: 'mkfs', 1: 'warm0', 2: 'warm0', 3: 'first'}
    holes, rec = [], []
    u = 3
    gap = 0
    def publish(kind):
        nonlocal u
        u += 1; events[u] = 'root'; kinds[u] = kind
    def crash_and_recover():
        nonlocal u, gap
        u += 1; events[u] = 'crash'; kinds[u] = 'crash'; holes.append(u)
        publish('row'); covered = {dev(u)}
        while covered != {0, 1}:
            publish('warm'); covered.add(dev(u))
        rec.append(u); gap = 0
    if schedule[0] in ('front', 'back'):
        pos, k = schedule
        first = FRONT_FIRST if pos == 'front' else P + 4
        while len(holes) < k:
            if u + 1 >= first and (not holes or gap >= GAP):
                crash_and_recover()
            else:
                publish('work'); gap = gap + 1 if holes else 0
    else:
        _, pos, m = schedule
        base = FRONT_FIRST if pos == 'front' else P + 4
        targets = [base + j * P for j in range(m)]
        for t in targets:
            while u + 1 < t:
                publish('work'); gap += 1
            assert u + 1 == t and (not holes or gap >= GAP), (S, schedule, t, u, gap)
            crash_and_recover()
    return events, holes, rec, kinds

def run(S, schedule, end=None):
    P = 3 * S
    events, holes, rec, kinds = history(S, schedule)
    last = max(events)
    if end is None:
        end = (holes[-1] + P + 2) if holes else last
    u = last
    while u < end:
        u += 1; events[u] = 'root'; kinds[u] = 'work'
    ring = {}
    units = {0: None}
    live_tt = 0
    pins = {}            # 槽 -> 钉住的 txg（这一次挂载那一刻读回全零、还没写掉）
    mount_first = {h + 1 for h in holes}
    rows = []
    for t in range(0, end + 1):
        if t not in (1, 2) and t > 0:
            units[live_tt] = t; units[t] = None; live_tt = t
        if t in mount_first:
            pins = {}
            for slot in range(P):
                if slot not in ring:
                    cands = [x for x in range(0, t) if x % P == slot]
                    if cands:
                        pins[slot] = min(cands)
        if events[t] == 'root':
            ring[t % P] = t
            if t % P in pins and t >= max(mount_first_le(t, mount_first), 0):
                del pins[t % P]
        if events[t] != 'root' or t < 3:
            continue
        A = set(ring.values())
        floor = min(A | set(pins.values()))
        withheld = {a for a, r in units.items() if r is None or r > floor}
        referenced = {a for a, r in units.items() if r is None or any(a <= x < r for x in A)}
        q1a = len(withheld - referenced)
        oldest = min(A)
        miss = sum(1 for v in range(oldest, t + 1) if v not in A)
        live = sum(1 for h in holes if oldest <= h <= t and h not in A and ring.get(h % P, -1) < h)
        measured = (t == 3 or kinds.get(t) == 'work' or t in rec)
        rows.append(dict(t=t, floor=floor, lag=t - floor, q1a=q1a, h_que=miss, h_dong=live,
                         pinned=len(pins), measured=measured))
    fits = all(not (a % P == b % P and abs(a - b) < P) for a, b in itertools.combinations(holes, 2)) \
        and all((h - P) not in holes for h in holes if h >= P)
    return rows, holes, rec, fits, end

def mount_first_le(t, mount_first):
    c = [x for x in mount_first if x <= t]
    return max(c) if c else 0

def k_full(S, pos):
    k = 1
    while True:
        _, holes, _, fits, _ = run(S, (pos, k + 1))
        if not fits:
            return k
        k += 1

def summary(S, label, schedule):
    rows, holes, rec, fits, end = run(S, schedule)
    obs = [r for r in rows if r['measured']]
    peak = max(r['q1a'] for r in obs)
    positive = sum(1 for r in obs if r['q1a'] > 0)
    print(f"S={S} family={label} holes={holes} recovery_ends={rec} end={end} fits={fits} "
          f"observations={len(obs)} q1a_tt_peak={peak} q1a_tt_positive_points={positive} q1a_tt_final={obs[-1]['q1a']} "
          f"lag_peak={max(r['lag'] for r in obs)} missing_txgs_peak={max(r['h_que'] for r in obs)} live_holes_peak={max(r['h_dong'] for r in obs)} "
          f"pinned_slots_peak={max(r['pinned'] for r in obs)}")
    return rows

def families(S):
    kf = k_full(S, 'front'); kb = k_full(S, 'back')
    out = [(f"Hh-front-k{k}", ('front', k)) for k in range(1, kf + 1)]
    out += [(f"Hh-back-k{k}", ('back', k)) for k in range(1, kb + 1)]
    out += [(f"Hc-front-n{n}", ('front', n)) for n in (2 * kf, 3 * kf)]
    out += [(f"Hs-{pos}-m{m}", ('slot', pos, m)) for pos in ('front', 'back') for m in (1, 2, 3)]
    return kf, kb, out

if __name__ == '__main__':
    dump = '--dump' in sys.argv
    for S in (4, 8, 16):
        kf, kb, fams = families(S)
        print(f"S={S} largest_front_hole_count_that_fits={kf} largest_back_hole_count_that_fits={kb}")
        for label, sch in fams:
            rows = summary(S, label, sch)
            if dump:
                for r in rows:
                    if r['measured']:
                        print(f"DUMP S={S} family={label} txg={r['t']} floor={r['floor']} lag={r['lag']} h_que={r['h_que']} "
                              f"h_dong={r['h_dong']} pinned={r['pinned']} q1a_tree_table_lower_bound={r['q1a']}")
