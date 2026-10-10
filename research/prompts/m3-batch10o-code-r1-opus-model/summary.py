#!/usr/bin/env python3
"""每份日志一行：段数、红的段数、抬 F/上限那一遍新类仍坏顶上过的段数（counters[0]>0 或 [3]>0）、其中被抛弃那一半（[1]>0）、
回退不拒臂顶上过的段数（[2]>0）、回退 Ok / Err 段数、ENOSPC 拒的段数。用法：summary.py <日志…>"""
import re, sys
for path in sys.argv[1:]:
    rows = [l for l in open(path, encoding='utf-8') if l.startswith('OPUSI1-RUN')]
    n = len(rows); red = sum(' red=true ' in l for l in rows)
    def c(l):
        m = re.search(r'counters=\[(\d+), (\d+), (\d+), (\d+)\]', l); return [int(x) for x in m.groups()]
    pushed = sum(c(l)[0] > 0 or c(l)[3] > 0 for l in rows)
    pushed_a = sum(c(l)[1] > 0 for l in rows)
    t2 = sum(c(l)[2] > 0 for l in rows)
    rok = sum(' rollback=Ok' in l for l in rows); rerr = sum(' rollback=Err' in l for l in rows)
    nospace = sum('NoSpace' in l for l in rows)
    print(f"SUMMARY {path.split('/')[-1]} segments={n} red={red} raise_or_ceiling_pushed={pushed} abandoned_pushed={pushed_a} t2_pushed={t2} rollback_ok={rok} rollback_err={rerr} nospace_refused={nospace}")
