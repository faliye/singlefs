#!/usr/bin/env python3
"""把 output-all.txt 里的计数排成报告用的几张表（只读、不算新数）。用法：python3 summarize.py output-all.txt"""
import sys
from collections import defaultdict

counts = {}
for line in open(sys.argv[1], encoding='utf-8'):
    if '-example ' in line or ' key=' not in line:
        continue
    key, value = line.split(' key=', 1)[1].rsplit(' value=', 1)
    counts[tuple(key.split('/'))] = int(value)

ARMS = ['TRUTH6', 'DING', 'DING_DROPLAST', 'YI', 'BING_R1', 'T1_STRICT', 'T1_OPT', 'T1_TX_STRICT', 'T1_TX_OPT', 'FP_LAST',
        'IDX', 'IDXN', 'WALK', 'CUR_IMPL']
METRICS = ['states', 'outcome_differs', 'head_differs_vs_DING', 'unverified_install', 'read_error_truth_reads',
           'durability_violations_truth_holds', 'durability_holds_truth_violates']


def arm_table(family, rule='prefix'):
    print(f'## {family} ({rule})')
    print('| arm | ' + ' | '.join(METRICS) + ' |')
    print('|---|' + '---|' * len(METRICS))
    for arm in ARMS:
        if (family, arm, rule, 'states') not in counts:
            continue
        print(f'| {arm} | ' + ' | '.join(str(counts.get((family, arm, rule, metric), 0)) for metric in METRICS) + ' |')
    print()


for family in ('L0', 'LONG', 'L1-ring', 'L1-root', 'L1-root+ring', 'L1U', 'M2', 'SW-mount', 'SW-root', 'SW-mem', 'SW-ringmax'):
    arm_table(family)
arm_table('L1-root+ring', 'ringmax')

print('## RB (prefix recovery rule; columns: uncommitted_differs / record_overwritten / unit_reused / head_or_other; committed_wrong)')
print('| arm | counter rule | isolation | differs | overwritten | reused | head/other | committed_wrong | uncommitted states |')
print('|---|---|---|---|---|---|---|---|---|')
for arm in ['TRUTH6', 'DING', 'YI', 'BING_R1', 'T1_TX_STRICT', 'T1_TX_OPT', 'FP_LAST', 'IDXN', 'WALK', 'CUR_IMPL']:
    for rule in ('old', 'mount', 'ringmax'):
        for isolation in ('shadow', 'named', 'reach'):
            base = ('RB', arm, rule, isolation)
            cells = [counts.get(base + ('uncommitted_differs_from_no_rollback',), 0),
                     counts.get(base + ('cause', 'baseline_record_overwritten'), 0),
                     counts.get(base + ('cause', 'baseline_unit_reused'), 0),
                     counts.get(base + ('cause', 'head_or_other'), 0),
                     counts.get(base + ('committed_wrong',), 0),
                     counts.get(base + ('uncommitted_states',), 0)]
            print(f'| {arm} | {rule} | {isolation} | ' + ' | '.join(map(str, cells)) + ' |')
print()

print('## WRAP (loops before the file is lost; -1 = never within 3 × ring)')
rows = defaultdict(dict)
for key, value in counts.items():
    if key[0] == 'WRAP':
        rows[(key[2], key[3], key[4])][key[1]] = value
print('| case | arm | rule | ring12 | ring24 | ring48 | ring96 |')
print('|---|---|---|---|---|---|---|')
for (case, arm, rule), by_ring in sorted(rows.items()):
    print(f'| {case} | {arm} | {rule} | ' + ' | '.join(str(by_ring.get(f'ring{ring}')) for ring in (12, 24, 48, 96)) + ' |')
