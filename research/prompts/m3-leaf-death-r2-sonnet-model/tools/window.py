#!/usr/bin/env python3
"""按 (读法, 政策, 树) 数 m 为正的扫描：有多少落在 unlink 途中（意图到最后一批之间）、有多少落在外面。
用法：window.py 产物… 。输出每个 (读法, 政策, 树) 一行，只列 T / TS / TSZ / FSZ / C1Z / C2HoldZ 这几种读法；
末尾一行数「m 扫描落在 unlink 外面」的格数。"""
import collections, sys

def parse(line):
    d = {}
    for t in line.split()[1:]:
        if '=' in t:
            k, v = t.split('=', 1)
            d[k] = v
    return d

readers = {'T', 'TS', 'TSZ', 'FSZ', 'C1Z', 'C2HoldZ'}
tot = collections.defaultdict(lambda: [0, 0, 0])
share = collections.defaultdict(lambda: [0, 0])  # (几何, 政策, 读法) → [inode 树上 m 为正的扫描数, 发布数]
publishes = collections.defaultdict(int)
for path in sys.argv[1:]:
    for line in open(path, encoding='utf-8'):
        if 'name=leaf_counts' in line:
            d = parse(line)
            publishes[(d['geometry'], d['policy'])] += int(d['publishes'])
            continue
        if 'name=leaf_cell' not in line:
            continue
        d = parse(line)
        if d['tree'] == 'inode' and d['reader'] in ('T', 'TSZ', 'FSZ'):
            share[(d['geometry'], d['policy'], d['reader'])][0] += int(d['m_positive_scans'])
        if d['reader'] not in readers:
            continue
        t = tot[(d['reader'], d['policy'], d['tree'])]
        t[0] += int(d['m_positive_scans'])
        t[1] += int(d['m_positive_scans_in_unlink'])
        t[2] += int(d['r_positive_scans'])
outside = 0
for key in sorted(tot):
    m, inside, r = tot[key]
    outside += (m != inside)
    print(f'{key[0]} {key[1]} {key[2]} m_scans={m} m_scans_in_unlink={inside} r_scans={r}')
print(f'cells={len(tot)} cells_with_m_scans_outside_unlink={outside}')
for key in sorted(share):
    print(f'window_share geometry={key[0]} policy={key[1]} reader={key[2]} inode_tree_m_positive_scans={share[key][0]} publishes={publishes[(key[0], key[1])]}')
