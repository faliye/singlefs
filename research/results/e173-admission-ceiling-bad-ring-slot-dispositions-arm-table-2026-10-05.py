#!/usr/bin/env python3
# E173 两段产物按臂分的位置数表（只读 research/results 下的产物，不读装置）：
# 第一段：每个 (持续方式, 池, 保留池读法, Q1 臂) 上各量「> 0 的位置数」之和（位置总数 = 四个历史族 × e/g 两种 × c_max 两档 × 3S − 1 的和）；
# 第二段：z 族与 H5，每个 (范围, 持续方式, 池, 保留池读法, Q1 臂, Q2 臂) 上同样的位置数，另加两条 Q2 臂逐位置相比的位置数。
import collections, sys
DATE = '2026-10-05'
PREFIX = '/home/fy5090/code/singlefs/research/results/e173-admission-ceiling-bad-ring-slot-dispositions'

def parse(line):
    return dict(token.split('=', 1) for token in line.strip().split(' ')[1:] if '=' in token)

def positive_positions(row, name):
    # 聚合写法：最大@位置/最小/量>0的位置数/位置总数
    return int(row[name].split('@')[1].split('/')[2]) if name in row else 0

QUANTITIES = ['extra_rejected', 'all_enospc', 'crossed', 'over_truth', 'lost_total', 'empties', 'empty_refusals', 'cap_blocked',
              'rej_budget', 'rej_floor_at_ceiling', 'rej_raise_no_placement', 'rej_empty_no_placement', 'rej_own_error']

def table(segment_file, key_of):
    totals = collections.defaultdict(collections.Counter)
    for line in open(segment_file, encoding='utf-8'):
        if 'name=cell' not in line:
            continue
        row = parse(line)
        counter = totals[key_of(row)]
        counter['positions'] += int(row['positions'])
        for name in QUANTITIES:
            counter[name] += positive_positions(row, name)
    return totals

def show(title, totals):
    print(title)
    print('key | positions | ' + ' '.join(QUANTITIES))
    for key in sorted(totals):
        counter = totals[key]
        print(key, counter['positions'], *[counter[name] for name in QUANTITIES])

show('第一段（e、g 读坏）：每个 (持续方式, 池, 保留池读法, Q1 臂) 上各量 > 0 的位置数之和',
     table(f'{PREFIX}-segment-one-{DATE}.out', lambda row: (row['persistence'], row['pool'], row['reserve'], row['q1'])))
second = f'{PREFIX}-segment-two-{DATE}.out'
show('第二段（z 与 H5）：每个 (范围, 持续方式, 池, 保留池读法, Q1 臂, Q2 臂) 上各量 > 0 的位置数之和',
     table(second, lambda row: ('H5' if row['family'] == 'H5' else 'z', row['persistence'], row['pool'], row['reserve'], row['q1'], row['q2'])))
pairs = collections.defaultdict(collections.Counter)
for line in open(second, encoding='utf-8'):
    if 'name=q2_pair' not in line:
        continue
    row = parse(line)
    counter = pairs[('H5' if row['family'] == 'H5' else 'z', row['persistence'], row['pool'], row['reserve'], row['q1'])]
    for name in ['positions', 'identical_positions', 'not_merged_crossed_more', 'merged_crossed_more', 'not_merged_lost_more', 'merged_lost_more',
                 'positions_merged_rejects_more', 'positions_merged_is_all_enospc_against_not_merged']:
        counter[name] += int(row[name])
print('第二段：两条 Q2 臂逐位置相比（同一 Q1 臂下）的位置数之和')
print('key | positions identical_positions not_merged_crossed_more merged_crossed_more not_merged_lost_more merged_lost_more positions_merged_rejects_more positions_merged_is_all_enospc_against_not_merged')
for key in sorted(pairs):
    counter = pairs[key]
    print(key, *[counter[name] for name in ['positions', 'identical_positions', 'not_merged_crossed_more', 'merged_crossed_more', 'not_merged_lost_more', 'merged_lost_more', 'positions_merged_rejects_more', 'positions_merged_is_all_enospc_against_not_merged']])
