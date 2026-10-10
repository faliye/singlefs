#!/usr/bin/env python3
"""m3-batch10z-no-code-r1 攻方：把 S3 行里红了的那几段按「红之前有没有卸载再挂 / 崩溃再挂」与故障组、R 分组数。用法：classify_red.py <日志…>"""
import re, sys, collections
c = collections.Counter()
for path in sys.argv[1:]:
    for line in open(path, errors='replace'):
        if 'S3 label' not in line or 'hit=true' not in line: continue
        line = line[line.index('S3 label'):]
        f = dict(re.findall(r'(\w+)=([^ ]*)', line))
        step = int(re.match(r'(\d+)', f['first_red']).group(1))
        before = f['actions'][:step]
        remount = 'remount-before-red' if ('u' in before or 'c' in before) else 'no-remount'
        c[(remount, f['faults'], f['r'], f['persist'])] += 1
for k, v in sorted(c.items()):
    print(f"RED {k[0]} faults={k[1]} r={k[2]} persist={k[3]} n={v}")
