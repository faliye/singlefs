import e139_port as m
from collections import Counter
C = Counter()
orig_ul = m.Sim.upper_limit
orig_dt = m.Sim.disposal_target
def both(self, fn, *a):
    p2 = self.cfg.p2
    self.cfg.p2 = False; x = fn(self, *a)
    self.cfg.p2 = True; y = fn(self, *a)
    self.cfg.p2 = p2
    return x, y
def ul(self):
    x, y = both(self, orig_ul)
    C['upper_limit_calls'] += 1
    if x != y:
        C['upper_limit_differs_with_p2'] += 1
    return orig_ul(self)
def dt(self, goal):
    x, y = both(self, orig_dt, goal)
    C['disposal_target_calls'] += 1
    if x != y:
        C['disposal_target_differs_with_p2'] += 1
    return orig_dt(self, goal)
m.Sim.upper_limit = ul
m.Sim.disposal_target = dt
cfg = m.Cfg(gate='window')
for v in m.VARIANTS:
    for S in (4, 16):
        for seed in range(8):
            m.run_task((cfg, v, S, seed))
print(dict(C))
