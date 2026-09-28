#!/usr/bin/env python3
"""变异表每条原文在源文件里命中几次（\\n 还原成换行）；只打命中数不是 1 的。"""
import sys
source = open(sys.argv[1], encoding='utf-8').read()
bad = 0
rows = 0
for line_number, line in enumerate(open(sys.argv[2], encoding='utf-8'), 1):
    line = line.rstrip('\n')
    if not line or line.startswith('#'):
        continue
    parts = line.split('\t')
    rows += 1
    name, original = parts[0], parts[1].replace('\\n', '\n')
    count = source.count(original)
    replacement = parts[2].replace('\\n', '\n') if len(parts) > 2 else None
    if count != 1:
        bad += 1
        print(f'line {line_number} {name}: hits={count}')
print(f'rows={rows} bad={bad}')
