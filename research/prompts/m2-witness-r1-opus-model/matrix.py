#!/usr/bin/env python3
"""每臂每边：按段列「回退成立的崩溃状态里 minF=1 / =2 / =3 / >3 各几个」与「回退成立但新实例一条根都没落盘」几个。"""
import re, sys, glob, collections
for path in sorted(glob.glob(sys.argv[1] + '/w-targeted-*.out')):
    arm = path.split('/')[-1][len('w-targeted-'):-4]
    for edge in ('E1', 'E2'):
        stream = {}
        cells = collections.defaultdict(collections.Counter)
        for line in open(path):
            m = re.match(r'STREAM %s .* seg=(\d+) \[(.*)\]' % edge, line)
            if m:
                stream[m.group(1)] = m.group(2)
            if not line.startswith('CRASH %s ' % edge):
                continue
            seg = re.search(r'seg=(\w+)/', line).group(1)
            minf = re.search(r'minF=(\S+)', line).group(1)
            if 'rb0=true' not in line:
                cells[seg]['notrb'] += 1
                continue
            cells[seg]['F' + minf] += 1
            if 'new_root_durable=false' in line:
                cells[seg]['rb_no_new_root'] += 1
            if 'torn=' in line and minf in ('1', '2'):
                cells[seg]['torn_F' + minf] += 1
        print('== %s %s' % (arm, edge))
        for seg in sorted(cells, key=lambda s: (s == 'done', int(s) if s.isdigit() else 99)):
            c = cells[seg]
            print('  seg=%-4s %-60s %s' % (seg, stream.get(seg, '(done)')[:60], ' '.join('%s=%d' % kv for kv in sorted(c.items()))))
