import e139_port as m
from collections import Counter
C = Counter()
orig = m.Sim.ceiling_roots
def patched(self):
    roots = self.valid_roots_with_disk()
    if not roots:
        return []
    anchor = max(r[0] for _, r in roots)
    inwin = [(d, r) for d, r in roots if self.in_window(r[0], anchor, self.ring_depth)]
    kept = [(d, r) for d, r in inwin if r[0] >= self.floor_carried]
    C['calls'] += 1
    if len(kept) != len(inwin):
        C['p2_filter_removes_some'] += 1
        # 去掉之后上限是否变
        C['p2_removed_roots_total'] += len(inwin) - len(kept)
    return kept if self.cfg.p2 else inwin
m.Sim.ceiling_roots = patched
# p4：记录分歧发生在哪种根上
orig_flags = m.Sim.nonempty_flags
def flags(self, roots):
    truth = {r[0]: (r[0] in self.nonempty) for r in roots}
    srt = sorted(roots, key=lambda r: (r[0], r[1]))
    prev = None
    for i, r in enumerate(srt):
        k = self.content_key(r)
        clause = (i == 0) or (k != prev)
        prev = k
        if clause != truth[r[0]]:
            C['p4_disagree_oldest' if i == 0 else 'p4_disagree_not_oldest'] += 1
            if i == 0:
                C['p4_disagree_oldest_clause_nonempty_truth_empty' if clause else 'p4_disagree_oldest_other'] += 1
    return orig_flags(self, roots)
m.Sim.nonempty_flags = flags
cfg = m.Cfg(gate='window', p2=False)
tasks = [(cfg, v, S, seed) for v in m.VARIANTS for S in (4, 16) for seed in range(8)]
for t in tasks:
    m.run_task(t)
print(dict(C))
