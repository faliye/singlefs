#!/usr/bin/env python3
"""把 w-targeted 的 CRASH 行按 (臂, 边, 段, 是否撕裂, minF) 归并，列 minF ≤ 2 的崩溃位置。"""
import re, sys, glob, collections
for path in sorted(glob.glob(sys.argv[1] + '/w-targeted-*.out')):
    arm = path.split('/')[-1][len('w-targeted-'):-4]
    groups = collections.Counter()
    shift = collections.Counter()
    for line in open(path):
        if not line.startswith('CRASH '):
            continue
        edge = line.split()[1]
        seg = re.search(r'seg=(\S+)(?: \[([^\]]*)\])?', line)
        torn = re.search(r'torn=(\S+)', line)
        minf = re.search(r'minF=(\S+)', line).group(1)
        rb0 = 'rb0=true' in line
        newroot = 'new_root_durable=true' in line
        key = (edge, seg.group(1), seg.group(2), 'torn:' + torn.group(1).split(':')[0] if torn else '-', minf)
        if rb0 and minf in ('1', '2'):
            groups[key] += 1
        if rb0 and not newroot:
            shift[(edge, seg.group(1))] += 1
    print('==', arm)
    for key, n in sorted(groups.items()):
        print('  minF<=2', *key, 'states=%d' % n)
    for key, n in sorted(shift.items()):
        print('  rolled_back_without_new_root', *key, 'states=%d' % n)
