#!/usr/bin/env python3
"""汇总 s1grid 日志（不看回退）：按 (起点, 槽类, 图案, red, 挂载后推进有没有被拒, 类) 数。用法：agg2.py 日志…"""
import re, sys, collections
rows = collections.Counter(); sample = {}
for path in sys.argv[1:]:
    for line in open(path):
        if not line.startswith('OPUSI1-RUN'): continue
        sp = re.search(r' sp=(\S+)', line).group(1)
        plan = re.search(r' plan=(\S+)', line).group(1)
        cls = plan.split('@')[0]
        pat = re.search(r'-trig\d-(\w)-', plan).group(1)
        red = re.search(r' red=(\w+)', line).group(1)
        ref = 'refused' if 'refused_at=' in line else 'no-refusal'
        cl = re.search(r' classes=(\{[^}]*\})', line)
        cl = cl.group(1) if cl else '?'
        key = (sp, cls, 'pat='+pat, 'red='+red, ref, cl)
        rows[key] += 1
        if (red == 'true' or ref == 'refused') and key not in sample:
            sample[key] = line[-700:].strip()
for k, n in sorted(rows.items()): print(n, *k)
for k, v in sample.items(): print('SAMPLE', *k, '::', v)
