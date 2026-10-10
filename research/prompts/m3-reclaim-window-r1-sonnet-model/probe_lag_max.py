import e139_port as m, k1_world as k
from collections import Counter
cfg = m.Cfg(gate='window', recover='replay', failure='resend')
C = Counter(); worst = 0
for double in (False, True):
  for S in (4, 16):
    for seed in range(8):
        for hw in (1, 2, 3):
            sim = m.Sim(cfg, S, seed); sim.prefill(0.70)
            ev = m.schedule_event_window(seed, sim.ring_depth)
            for _ in range(ev + hw - 1): sim.run_user_window(True, False)
            sim.run_user_window(True, True); sim.crash_and_recover()
            if double:
                sim.run_user_window(True, True); sim.crash_and_recover()
            for act in ['row', 'warm', 'warm', 'warm'] + ['win'] * 6:
                if act == 'warm' and k.covered_both_disks(sim): continue
                if act in ('row', 'warm'): sim.empty_publication()
                else: sim.run_user_window(True, False)
                for lost in (0, 1):
                    s = sim.clone(); s.disk_lost[lost] = True
                    ch = s.newest_valid_root()
                    lag = s.max_published - ch[0]
                    C[lag] += 1; worst = max(worst, lag)
print('lag N - chosen distribution (points):', dict(sorted(C.items())), 'max', worst, '3S min', 12)
