#!/usr/bin/env python3
"""按种子切开追踪（每个种子的行连续；种子之间以第一次崩溃段号回落为界），每个种子里按（第一次崩溃段号、写行那一段的写数 n）算一次独立抽样，
数每个 n 的抽样次数、命中次数与被抽中的写在写行写里的位置。"""
import re, sys, collections
pat = re.compile(r"first_seg=(\d+) .*?mount_segments=([\d+]*) row_publish_writes=(\d+) second_seg=(\d+) second_kinds=(\[.*?\]) second_bits=([01]*) phase=row_publish .*?c670_illegal=(\d+)")
trials = collections.Counter(); hits = collections.Counter(); picks = collections.Counter()
seeds = 0; seeds_hit = 0
for path in sys.argv[1:]:
    seed_trials = None; last_fs = -1; seed_hit = False
    def flush():
        global seeds, seeds_hit
        if seed_trials is None: return
        seeds += 1; seeds_hit += seed_hit
        for (fs, n), (s2, kinds, bits, ill) in seed_trials.items():
            trials[n] += 1; hits[n] += ill > 0
            picks[(n, s2, kinds, bits.index('0') if '0' in bits else -1)] += 1
    for line in open(path):
        m = pat.search(line)
        if not m: continue
        fs, segs, n, s2, kinds, bits, ill = int(m.group(1)), m.group(2), int(m.group(3)), int(m.group(4)), m.group(5), m.group(6), int(m.group(7))
        if fs < last_fs:
            flush(); seed_trials = None; seed_hit = False
        if seed_trials is None: seed_trials = {}
        last_fs = fs
        short = "".join({"unit_write":"U","journal_record":"J","root_record_fua":"R","system_configuration_slot":"S"}.get(x.strip().strip('"'),"?") for x in kinds.strip("[]").split(",") if x.strip())
        prev = seed_trials.get((fs, n))
        if prev is None:
            seed_trials[(fs, n)] = (s2, short, bits, ill)
        else:
            assert prev[:3] == (s2, short, bits), ("同一个（段号, n）抽到了不同的二次崩溃", fs, n, prev, (s2, short, bits))
            seed_trials[(fs, n)] = (s2, short, bits, max(prev[3], ill))
        seed_hit = seed_hit or ill > 0
    flush()
print(f"seeds={seeds} seeds_hit={seeds_hit}")
for n in sorted(trials):
    print(f"n={n} trials={trials[n]} hits={hits[n]} rate={hits[n]/trials[n]:.3f} 7/(4n)={7/(4*n):.3f}")
seg_counts = collections.Counter()
for (n, s2, kinds, first_withheld), c in picks.items():
    seg_counts[(n, s2, kinds)] += c
for key in sorted(seg_counts):
    print("picked segment", key, seg_counts[key])
