#!/usr/bin/env python3
"""G6 算术核：今天的触发在 publish_user_change 返回之前跑至多一批（mounted_session.rs 第 1927-1936 行），
所以前台一次改动的返回要多等「一批 + 收尾 discard」。从 E185 第 2 次重跑第一段产物取每个点一批（kind=batch 或 final_batch）
调用的挂钟、CPU、写字节，报中位与最大；再算这一批按写带宽上限 W 限速时要多长（写字节 ÷ W），
看写带宽上限的默认值能不能把这段等压下去。用法：python3 -I g6_inline_batch_latency.py <wall-clock 产物> <W 字节每秒，逗号分隔>"""
import re, sys, statistics
from collections import defaultdict

def fields(line):
    return dict(m.groups() for m in re.finditer(r'(\w+)=(\S+)', line))

path, caps = sys.argv[1], [float(x) for x in sys.argv[2].split(',')]
groups = defaultdict(list)
for l in open(path):
    if not l.startswith('E7RESULT name=call '):
        continue
    f = fields(l)
    if f.get('injection') != 'none' or f['kind'] not in ('batch', 'final_batch'):
        continue
    groups[(f['arm'], f['fraction'], f['pool_data_units'], f['batch_units'])].append(
        (int(f['wall_nanoseconds']), int(f['cpu_nanoseconds']), int(f['write_bytes'])))
for key, vals in sorted(groups.items(), key=lambda kv: (kv[0][0], float(kv[0][1]), int(kv[0][2]), int(kv[0][3]))):
    arm, fr, P, K = key
    walls = [v[0] / 1e6 for v in vals]; cpus = [v[1] / 1e6 for v in vals]; wb = [v[2] for v in vals]
    paced = ' '.join(f'paced_ms_at_{int(c)}={statistics.median(wb)/c*1e3:.3f}' for c in caps)
    print(f'name=g6_batch arm={arm} f={fr} P={P} K={K} batches={len(vals)} wall_ms_median={statistics.median(walls):.3f} wall_ms_max={max(walls):.3f} cpu_over_wall_median={statistics.median(c/w for c, w in zip(cpus, walls)):.4f} write_bytes_median={int(statistics.median(wb))} {paced}')
