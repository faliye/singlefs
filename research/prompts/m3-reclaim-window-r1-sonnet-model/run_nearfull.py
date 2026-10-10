import sys, re
import e139_port as m
from multiprocessing import Pool

def drain_task(a):
    cfg, S, cp, seed = a
    mc, dm = cp
    pre = m.drain_prefix(cfg, S, mc, dm, seed)
    out = {}
    if pre is None:
        return out
    for pat in m.drain_patterns(8):
        r = m.run_drain_case(pre, pat)
        t = out.setdefault(len(pat), m.Metrics())
        m.add_metrics(t, r)
        out.setdefault(('cases', len(pat)), 0)
        out[('cases', len(pat))] += 1
    return out

def broken_task(a):
    cfg, S, cp, seed = a
    mc, dm = cp
    return m.run_broken_slot(cfg, S, mc, dm, seed)

def main():
    kv = sys.argv[1] if len(sys.argv) > 1 else ''
    kw = {}
    for p in filter(None, kv.split(',')):
        k, v = p.split('=')
        kw[k] = (v == 'True') if v in ('True', 'False') else v
    cfg = m.Cfg(**kw)
    tasks = [(cfg, S, cp, seed) for S in (4, 16) for cp in m.COST_PAIRS for seed in range(8)]
    with Pool(24) as pool:
        dres = pool.map(drain_task, tasks)
        bres = pool.map(broken_task, tasks)
    agg = {}
    for (cfg_, S, cp, seed), out in zip(tasks, dres):
        for k, v in out.items():
            if isinstance(k, tuple):
                agg.setdefault((S, cp, 'cases', k[1]), 0)
                agg[(S, cp, 'cases', k[1])] += v
            else:
                t = agg.setdefault((S, cp, 'm', k), m.Metrics())
                m.add_metrics(t, v)
    print('# cfg', cfg.tag())
    for S in (4, 16):
        for cp in m.COST_PAIRS:
            for nf in (1, 2, 3):
                t = agg[(S, cp, 'm', nf)]
                print(f'name=drain_failure s={S} c0={cp[0]} c_max={cp[1]} failures={nf} cases={agg[(S,cp,"cases",nf)]} false_enospc_arm={t.false_enospc_arm} max_publications={t.max_pubs} stalls={t.stalls} fakes={t.fake} unrecoverable={t.unrec}')
    for S in (4, 16):
        for cp in m.COST_PAIRS:
            t = m.Metrics()
            for (cfg_, S2, cp2, seed), r in zip(tasks, bres):
                if S2 == S and cp2 == cp:
                    m.add_metrics(t, r)
            print(f'name=broken_slot s={S} c0={cp[0]} c_max={cp[1]} false_enospc_arm={t.false_enospc_arm} write_after_delete_failed={t.write_after_delete_failed} max_publications={t.max_pubs} stalls={t.stalls} fakes={t.fake} unrecoverable={t.unrec} injected={t.failures_injected}')
main()
