#!/usr/bin/env python3
"""K2 汇总：python3 k2agg.py <日志…>：按 (起点, 抬 F 那一遍有没有读坏过 S1 槽, 回退结局, 拒在哪一槽) 计数。"""
import re, sys
for f in sys.argv[1:]:
    c = {}
    for l in open(f, encoding='utf-8'):
        if not l.startswith('OPUSI1-RUN'):
            continue
        sp = re.search(r' sp=(\S+)', l).group(1)
        plan = re.search(r' plan=s1-V@(\d+)/(\d+)', l)
        reached = re.search(r'reached=(\S+)', l).group(1)
        m = re.search(r'rollback=(\S+)', l)
        rb = m.group(1) if m else 'skipped(user-write-refused)'
        slot = re.search(r'ring_slot:RootRingSlot\{region:(\d+),slot:(\d+)\}', rb)
        where = '-' if not slot else ('S1-slot' if plan and slot.groups() == plan.groups() else 'other-slot')
        kind = rb.split('(')[0] + ('' if rb.startswith('skipped') else '(' + re.sub(r'^txg\d+:?', '', rb.split('(', 1)[1])[:45])
        k = (sp, reached, kind, where)
        c[k] = c.get(k, 0) + 1
    print('K2AGG', f.rsplit('/', 1)[-1])
    for k, v in sorted(c.items()):
        print('  ' + ' '.join(k) + f' n={v}')
