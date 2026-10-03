#!/usr/bin/env python3
"""E165 第二段红例账（research/results/e165-...-segment2-red-examples.tsv）按变异汇总：
每个变异在合法历史族、对抗族、R 域的首红状态上「多出」(+) 的条款判据。只看首红那一个状态，是下限，不是整域合计。
用法：python3 ledger_by_criterion.py            （在仓根下跑）
"""
import collections
import sys

LEDGER = "research/results/e165-replay-bounded-enumeration-2026-10-01-segment2-red-examples.tsv"
LEGAL = {'L', 'L6', 'U_verify_on', 'U_verify_off', 'X', 'E1', 'E2', 'C', 'C6'}
ADVERSARIAL = {'A', 'A6', 'AC'}
CLAUSE = {'J1', 'J2', 'J3', 'J4', 'J5a', 'J5b', 'J6_premise_holds', 'J6_premise_fails',
          'A_structure', 'A_limit', 'AS_whole_table', 'AS_consecutive_gap_continues', 'AS_consecutive_gap_stops'}
LITERAL_WITHOUT_J4 = {'J1', 'J2', 'J3', 'J5a', 'J5b', 'J6_premise_holds', 'J6_premise_fails', 'A_structure', 'A_limit'}

rows = [line.rstrip('\n').split('\t') for line in open(LEDGER, encoding='utf-8') if not line.startswith('#')]
per = collections.OrderedDict()
for r in rows:
    variant, domain, diff = r[0], r[2], r[6]
    entry = per.setdefault(variant, {'legal': set(), 'adversarial': set(), 'R': set(), 'minus': set()})
    for item in diff.split(','):
        name = item[1:]
        if name not in CLAUSE:
            continue
        if item.startswith('-'):
            entry['minus'].add((domain, name))
            continue
        if domain in LEGAL:
            entry['legal'].add(name)
        elif domain in ADVERSARIAL:
            entry['adversarial'].add(name)
        elif domain == 'R':
            entry['R'].add(name)
print('变异\t合法历史族首红例里多出的条款判据\t对抗族\tR 域\t少出的（域:判据）')
for variant, e in per.items():
    print('%s\t%s\t%s\t%s\t%s' % (variant, ','.join(sorted(e['legal'])) or '-', ','.join(sorted(e['adversarial'])) or '-',
                                   ','.join(sorted(e['R'])) or '-', ','.join(sorted('%s:%s' % d for d in e['minus'])) or '-'))
inside = [v for v, e in per.items() if (e['legal'] | e['adversarial'] | e['R']) & LITERAL_WITHOUT_J4]
print('红例账里有首红例的变异 %d 个；首红例里多出 J1、J2、J3、J5、J6、A-结构、A-限 之一的 %d 个：%s' % (len(per), len(inside), ' '.join(sorted(inside))))
print('合法历史族首红例里多出 J1 的：%s' % ' '.join(v for v, e in per.items() if 'J1' in e['legal']))
print('合法历史族首红例里多出 J2 的：%s' % ' '.join(v for v, e in per.items() if 'J2' in e['legal']))
print('对抗族首红例里多出 A-结构 的：%s' % ' '.join(v for v, e in per.items() if 'A_structure' in e['adversarial']))
