#!/usr/bin/env python3
"""实现形的 K1 世界：挂载前一次发布崩在「记录已持久、根槽没持久」，可写挂载由记录施加出那一版（洞），
写行 + 暖机推到本实例的根覆盖每块盘，之后在每个发布点上崩溃 + 丢一块盘，在两种锚上数假候选：
  A_impl = 现行那一版的 txg（实现 mount.rs 第 6280 行回退候选集的锚）
  A_chk  = 幸存盘上最新的有效根的 txg（池级 checker 候选过滤的锚，walk.rs 第 8081 行）
端口的内核是 port_v3.py（已逐行复现 E139 产物的 w_activation、w_drain_failure、w_broken_slot 行）。
"""
import sys
import e139_port as m
from multiprocessing import Pool


def covered_both_disks(sim):
    seen = set()
    for d, r in sim.valid_roots_with_disk():
        if r[1] == sim.instance:
            seen.add(d)
    return len(seen) == m.DISKS


def remount(sim, crash_before_root_pending=True):
    """可写挂载：恢复（实现形，施加记录）。调用前最后一次发布崩在根槽之前（pending），或没有 pending。"""
    sim.crash_and_recover()


def count_fake_anchor(sim, anchor):
    floor = sim.candidacy_floor()
    return sum(1 for r in sim.valid_roots() if r[0] >= floor and sim.in_window(r[0], anchor, sim.ring_depth) and r[0] in sim.damaged)


def evaluate_loss(sim, lost):
    """在当前点崩溃（无 pending）并丢盘 lost，替换成实现形恢复；返回 (A_impl 假候选, A_chk 假候选, 择中的根是否被复用)。"""
    s = sim.clone()
    s.disk_lost[lost] = True
    chosen = s.newest_valid_root()
    if chosen is None:
        return (0, 0, 1)
    eff = s.unpublished_txg if s.unpublished_txg is not None else s.max_published
    chosen_damaged = int(chosen[0] in s.damaged)
    fake_impl = count_fake_anchor(s, eff)
    fake_chk = count_fake_anchor(s, chosen[0])
    return (fake_impl, fake_chk, chosen_damaged)


def scenario(args):
    cfg, S, seed, hole_window, double_hole = args
    sim = m.Sim(cfg, S, seed)
    sim.prefill(0.70)
    ev = m.schedule_event_window(seed, sim.ring_depth)
    for _ in range(ev):
        sim.run_user_window(True, False)
    for _ in range(hole_window - 1):
        sim.run_user_window(True, False)
    sim.run_user_window(True, True)          # 崩在根槽之前：记录已持久，版本 x 没有根
    x = sim.unpublished_txg
    sim.crash_and_recover()                  # 挂载 2：实现形（施加记录）
    if double_hole:
        sim.run_user_window(True, True)      # 挂载 2 的写行也崩在根槽之前
        sim.crash_and_recover()              # 挂载 3
    rows = []                                # (k, lost, fake_impl, fake_chk, chosen_damaged)
    k = 0
    # 动作序列：写行（空发布）、暖机空发布（至多 R 次，覆盖每块盘为止）、之后 6 个用户窗口
    actions = ['row', 'warm', 'warm', 'warm'] + ['win'] * 6
    for act in actions:
        if sim.halted:
            break
        if act == 'warm' and covered_both_disks(sim):
            continue
        if act in ('row', 'warm'):
            sim.empty_publication()
        else:
            sim.run_user_window(True, False)
        k += 1
        for lost in (0, 1):
            fi, fc, cd = evaluate_loss(sim, lost)
            rows.append((k, lost, fi, fc, cd))
    # 成本：洞之后继续 30 个用户窗口，量每个窗口开头的门槛与被扣住的已释放块数
    pins = []
    gates = []
    for _ in range(30):
        if sim.halted:
            break
        b = sim.reuse_bound()
        pins.append(sum(len(v) for g, v in sim.by_gen.items() if g > b))
        gates.append(b)
        sim.run_user_window(True, False)
    return (S, seed, hole_window, double_hole, x, rows, pins, gates)


def main():
    import argparse
    ap = argparse.ArgumentParser()
    ap.add_argument('--cfg', default='gate=window')
    ap.add_argument('--double', type=int, default=0)
    a = ap.parse_args()
    kw = {}
    for kv in filter(None, a.cfg.split(',')):
        k, v = kv.split('=')
        kw[k] = (v == 'True') if v in ('True', 'False') else v
    kw.setdefault('recover', 'replay')
    kw.setdefault('failure', 'resend')
    cfg = m.Cfg(**kw)
    tasks = [(cfg, S, seed, hw, bool(a.double)) for S in (4, 16) for seed in range(8) for hw in (1, 2, 3)]
    with Pool(24) as pool:
        res = pool.map(scenario, tasks)
    print('# cfg', cfg.tag(), 'double_hole', a.double)
    agg = {}
    for (S, seed, hw, dh, x, rows, pins, gates) in res:
        for (k, lost, fi, fc, cd) in rows:
            t = agg.setdefault((S, k), [0, 0, 0, 0, 0])   # 点数, 有 A_impl 假候选的点, 有 A_chk 假候选的点, A_chk 假候选总数, 择中的根被复用的点
            t[0] += 1
            t[1] += int(fi > 0)
            t[2] += int(fc > 0)
            t[3] += fc
            t[4] += cd
    for (S, k), t in sorted(agg.items()):
        print(f'S={S} point_after_publication={k} points={t[0]} points_with_fake_at_A_impl={t[1]} points_with_fake_at_A_chk={t[2]} fakes_at_A_chk={t[3]} chosen_root_damaged_points={t[4]}')
    for S in (4, 16):
        allpins = [p for r in res if r[0] == S for p in r[6]]
        pins_by_window = [[r[6][i] for r in res if r[0] == S and len(r[6]) > i] for i in range(30)]
        mean = sum(allpins) / max(1, len(allpins))
        print(f'S={S} cost: pinned_freed_blocks_mean={mean:.1f} max={max(allpins) if allpins else 0} windows_sampled={len(allpins)}')
        first = [sum(w) / len(w) for w in pins_by_window[:3]]
        print(f'S={S} cost: mean pinned at window 1..3 after the hole = {[round(x,1) for x in first]}')
    tot_fi = sum(t[1] for t in agg.values()); tot_fc = sum(t[2] for t in agg.values()); tot_pts = sum(t[0] for t in agg.values()); tot_cd = sum(t[4] for t in agg.values())
    print(f'# total points={tot_pts} with_fake_at_A_impl={tot_fi} with_fake_at_A_chk={tot_fc} chosen_root_damaged={tot_cd}')


if __name__ == '__main__':
    main()
