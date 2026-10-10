#!/usr/bin/env python3
"""把 out/ 里各步输出里确定性的那几类行汇成一份摘要（去掉耗时、按行排序）；FIRSTRED 与 CHECKERRED 的例子随线程先后变，不进摘要。"""
import os, re, sys

out = sys.argv[1]
keep = re.compile(r'^(COUNT|SUMMARY|PRESS |PRESSLEGEND|PRESSAGG|AGG |S4A |T7 |XLEAF |TOMB |CRASHCOUNT|  step #|STEP #|STEPLINE)')
lines = []
for name in sorted(os.listdir(out)):
    if not name.endswith('.out'):
        continue
    with open(os.path.join(out, name), encoding='utf-8') as handle:
        for line in handle:
            line = line.rstrip('\n')
            if not keep.match(line):
                continue
            if line.startswith('STEPLINE') and not name.startswith('show-'):
                continue
            line = re.sub(r' elapsed=[0-9.]+s', '', line)
            lines.append(f'{name}: {line}')
for line in sorted(lines):
    print(line)
