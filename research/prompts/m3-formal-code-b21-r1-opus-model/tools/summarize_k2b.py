#!/usr/bin/env python3
"""k2b-sweep.txt 按（续做结局、nlink、发布臂、建 1 个、建 0 个、覆盖写）分类计数，每类给前三个盘面。"""
import collections
import sys

rows = [line.strip() for line in open(sys.argv[1]) if line.startswith('slots=')]
counts = collections.Counter()
examples = collections.defaultdict(list)
for row in rows:
    if 'build-failed' in row:
        counts[('build-failed',)] += 1
        continue
    fields = dict(item.split('=', 1) for item in row.split(' ') if '=' in item)
    key = tuple(fields[name].split('|')[0][:32] for name in ('continuation', 'nlink', 'arm', 'create1', 'create0', 'overwrite'))
    counts[key] += 1
    examples[key].append('/'.join(fields[name] for name in ('slots', 'ow', 'units', 'batches')))
print(f'rows={len(rows)}')
for key, count in sorted(counts.items(), key=lambda item: (-item[1], item[0])):
    print(count, ' '.join(key), examples.get(key, [])[:3])
