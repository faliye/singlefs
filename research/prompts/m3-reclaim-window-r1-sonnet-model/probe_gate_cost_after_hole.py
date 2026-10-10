import e139_port as m, k1_world as k
from collections import defaultdict
cfg = m.Cfg(gate='window', recover='replay', failure='resend')
diff_by_k = defaultdict(list)   # k -> list of (G_window - G_alow, extra pinned blocks)
ring_by_k = defaultdict(list)
for S in (4, 16):
    for seed in range(8):
        for hw in (1, 2, 3):
            sim = m.Sim(cfg, S, seed); sim.prefill(0.70)
            ev = m.schedule_event_window(seed, sim.ring_depth)
            for _ in range(ev + hw - 1): sim.run_user_window(True, False)
            sim.run_user_window(True, True)
            sim.crash_and_recover()
            actions = ['row', 'warm', 'warm', 'warm'] + ['win'] * 6
            kk = 0
            for act in actions:
                if act == 'warm' and k.covered_both_disks(sim): continue
                if act in ('row', 'warm'): sim.empty_publication()
                else: sim.run_user_window(True, False)
                kk += 1
                gw = sim.reuse_bound()
                sim.cfg.gate = 'alow'; ga = sim.reuse_bound()
                sim.cfg.gate = 'ring'; gr = sim.reuse_bound()
                sim.cfg.gate = 'window'
                ex_a = sum(len(v) for g, v in sim.by_gen.items() if ga < g <= gw)
                ex_r = sum(len(v) for g, v in sim.by_gen.items() if gr < g <= gw)
                diff_by_k[(S, kk)].append((gw - ga, ex_a))
                ring_by_k[(S, kk)].append((gw - gr, ex_r))
for (S, kk), v in sorted(diff_by_k.items()):
    if kk > 8: continue
    r = ring_by_k[(S, kk)]
    print(f'S={S} after publication {kk}: n={len(v)} alow: gate_lower_by_txgs mean={sum(a for a,_ in v)/len(v):.2f} max={max(a for a,_ in v)} extra_pinned_blocks mean={sum(b for _,b in v)/len(v):.1f} max={max(b for _,b in v)} | ring: txgs mean={sum(a for a,_ in r)/len(r):.2f} max={max(a for a,_ in r)} extra_pinned mean={sum(b for _,b in r)/len(r):.1f} max={max(b for _,b in r)}')
