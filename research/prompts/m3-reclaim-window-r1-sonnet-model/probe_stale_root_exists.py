import e139_port as m, k1_world as k
cfg = m.Cfg(gate='window', recover='replay', failure='resend')
tot = 0; with_stale = 0; stale_txgs = []
lifetimes = []
for S in (4, 16):
    for seed in range(8):
        for hw in (1, 2, 3):
            sim = m.Sim(cfg, S, seed); sim.prefill(0.70)
            ev = m.schedule_event_window(seed, sim.ring_depth)
            for _ in range(ev + hw - 1): sim.run_user_window(True, False)
            sim.run_user_window(True, True)
            x = sim.unpublished_txg
            sim.crash_and_recover()
            for act in ['row', 'warm', 'warm', 'warm']:
                if act == 'warm' and k.covered_both_disks(sim): continue
                sim.empty_publication()
            n = sim.max_published
            stale = [r[0] for r in sim.valid_roots() if r[0] + sim.ring_depth <= n]
            tot += 1
            if stale: with_stale += 1
            # 洞里那条旧根是 x - 3S：确认
            slot_x = sim.ring[sim.slot_index(x)]
            lifetimes.append((S, x, slot_x[0] if slot_x else None, x - sim.ring_depth, n))
print('scenarios', tot, 'with a valid root outside the window after the mount', with_stale)
print('hole slot holds x-3S in', sum(1 for (S, x, held, exp, n) in lifetimes if held == exp), 'of', len(lifetimes))
print(lifetimes[:4])
