import e139_port as m
from multiprocessing import Pool
import sys
# B2：门槛取环里最旧有效根，候选集也不再按窗口滤（六处一起退回「环」）
m.Sim.in_cand_window = lambda self, txg: True
cfg = m.Cfg(gate='ring')
tasks = [(cfg, v, S, seed) for v in m.VARIANTS for S in (4, 16) for seed in range(8)]
with Pool(16) as p:
    res = p.map(m.run_task, tasks)
rows = m.aggregate(res)
f = sum(1 for v in rows.values() if v[0] > 0); u = sum(1 for v in rows.values() if v[1] > 0)
print('B2 activation family: rows', len(rows), 'rows_with_fakes', f, 'rows_with_unrecoverable', u, 'total_fakes', sum(v[0] for v in rows.values()), 'total_unrec', sum(v[1] for v in rows.values()))
