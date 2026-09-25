#!/usr/bin/env python3
"""每臂每边一行：逐段列「回退已成立的崩溃状态里最小的 minF」，这一段没有回退成立的状态记「-」；段名取写的种类。"""
import re, sys, glob, collections
order = lambda f: f
for path in sorted(glob.glob(sys.argv[1] + '/w-targeted-*.out')):
    arm = path.split('/')[-1][len('w-targeted-'):-4]
    for edge in sys.argv[2:]:
        names, best = {}, {}
        for line in open(path):
            m = re.match(r'STREAM %s .* seg=(\d+) \[(.*)\]' % edge, line)
            if m:
                kinds = sorted(set(re.sub(r'[0-9.]', '', k) for k in m.group(2).split(',')))
                names[m.group(1)] = '+'.join(kinds)
            if not line.startswith('CRASH %s ' % edge) or 'rb0=true' not in line:
                continue
            seg = re.search(r'seg=(\w+)/', line).group(1)
            minf = re.search(r'minF=(\S+)', line).group(1)
            value = 99 if minf == '>3' else int(minf)
            best[seg] = min(best.get(seg, 99), value)
        segs = sorted(names, key=int) + ['done']
        cells = []
        for seg in segs:
            v = best.get(seg)
            cells.append('%s:%s=%s' % (seg, names.get(seg, 'done'), '-' if v is None else ('>3' if v == 99 else v)))
        print(arm, edge, ' '.join(cells))
