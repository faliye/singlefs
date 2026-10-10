#!/usr/bin/env python3
"""r2-sweep 的产物：每个 (n0, 删序, 删多少%, 每删几个建一批) 一行，M3 均分借 / 刚好够 并排；末尾汇总。"""
import sys

def parse(line):
    d = {}
    for t in line.split()[1:]:
        if '=' in t:
            k, v = t.split('=', 1)
            d[k] = v
    return d

rows = {}
for path in sys.argv[1:]:
    for line in open(path, encoding='utf-8'):
        if 'name=leaf_counts' not in line:
            continue
        d = parse(line)
        key = (int(d['n0']), int(d['order']), int(d['delete']), int(d['creates_every']))
        rows.setdefault(key, {})[d['policy']] = d
print('n0 order delete% creates_every | borrows half/just | records_moved half/just | leaves_rewritten_in_removals half/just')
sums = dict(hb=0, jb=0, hm=0, jm=0, hr=0, jr=0)
cells_half_rewrites_more = 0
cells_half_moves_fewer_records = 0
for key in sorted(rows):
    h, j = rows[key]['M3half_dropped'], rows[key]['M3just_dropped']
    hb, jb = int(h['borrows']), int(j['borrows'])
    hm, jm = int(h['borrowed_records']), int(j['borrowed_records'])
    hr, jr = int(h['containers_rewritten_in_removals']), int(j['containers_rewritten_in_removals'])
    for name, value in (('hb', hb), ('jb', jb), ('hm', hm), ('jm', jm), ('hr', hr), ('jr', jr)):
        sums[name] += value
    cells_half_rewrites_more += hr > jr
    cells_half_moves_fewer_records += hm < jm
    print(*key, f'| {hb}/{jb} | {hm}/{jm} | {hr}/{jr}')
print('cells', len(rows), 'cells_where_half_rewrites_more_leaves_than_just', cells_half_rewrites_more, 'cells_where_half_moves_fewer_records_than_just', cells_half_moves_fewer_records)
print(' '.join(f'{k}={v}' for k, v in sums.items()))
