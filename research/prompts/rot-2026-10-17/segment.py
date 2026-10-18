#!/usr/bin/env python3
"""把候选表按组切段：一组不拆，每段约 TARGET 行（一组超过就整组一段）。输出 segments.tsv：段号、组号清单（逗号分隔，给 --groups 用）、行数。
用法：python3 segment.py 候选.tsv segments.tsv [TARGET]
"""
import sys, collections
table, out = sys.argv[1], sys.argv[2]
target = int(sys.argv[3]) if len(sys.argv) > 3 else 250
counts = collections.OrderedDict()
with open(table, encoding='utf-8') as handle:
    next(handle)
    for line in handle:
        group = line.split('\t', 1)[0]
        counts[group] = counts.get(group, 0) + 1
def key(g):
    import re
    m = re.fullmatch(r'([A-Z])(\d+)', g); return (m.group(1), int(m.group(2))) if m else (g, 0)
segments, current, acc = [], [], 0
for group in sorted(counts, key=key):
    n = counts[group]
    if current and acc + n > target:
        segments.append((current, acc)); current, acc = [], 0
    current.append(group); acc += n
if current: segments.append((current, acc))
with open(out, 'w', encoding='utf-8') as handle:
    handle.write('段\t组号\t行数\n')
    for index, (groups, lines) in enumerate(segments, 1):
        handle.write(f'S{index:02d}\t{",".join(groups)}\t{lines}\n')
print(f'  ✓ {len(segments)} 段，{sum(counts.values())} 行候选，{len(counts)} 个组；最大一段 {max(l for _, l in segments)} 行')
