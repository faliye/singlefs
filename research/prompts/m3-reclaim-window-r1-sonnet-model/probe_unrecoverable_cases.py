import e139_port as m
cfg = m.Cfg(gate='window')
seen = 0
orig = m.Sim.crash_and_recover
def patched(self):
    ch = self.newest_valid_root()
    if ch is not None and ch[0] in self.damaged and self.cfg.recover == 'chosen':
        global seen
        seen += 1
        if seen <= 6:
            print('chosen', ch[0], 'N(max published)', self.max_published, '3S', self.ring_depth, 'N-chosen', self.max_published - ch[0],
                  'floor_active', self.floor_active, 'floor_carried', self.floor_carried, 'in_window(N)', self.in_window(ch[0], self.max_published, self.ring_depth),
                  'lost', self.disk_lost, 'disk1_valid_txgs', sorted(r[0] for d, r in self.valid_roots_with_disk() if d == 1)[-6:])
    return orig(self)
m.Sim.crash_and_recover = patched
for v in ('skip2', 'skip3'):
    for seed in range(8):
        m.run_task((cfg, v, 4, seed))
print('unrecoverable events seen', seen)
