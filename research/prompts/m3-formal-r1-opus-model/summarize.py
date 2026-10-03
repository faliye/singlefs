#!/usr/bin/env python3
"""把 results.tsv 汇成报告里那几张表。用法：python3 summarize.py results.tsv campaign.log"""
import collections, csv, re, sys
rows = list(csv.DictReader(open(sys.argv[1]), delimiter='\t'))
ids, names, cell = [], {}, collections.defaultdict(dict)
for r in rows:
    if r['id'] not in names:
        ids.append(r['id']); names[r['id']] = r['name']
    cell[r['id']][(r['mode'], r['criterion'])] = r['min_n']
LEGAL = ['base', 'units', 'units2', 'stray', 'wrap', 'edge']
ADV = 'adv+PROBE_ADV_FOLLOW=monotone'
def first(i, crit, modes):
    best = None
    for m in modes:
        v = cell[i].get((m, crit), '-')
        if crit == 'F' and m == 'wrap':
            continue  # 原实现在 wrap 上 F 自己就红（在飞上限咬人），不算杀伤
        if v not in ('-', None) and (best is None or int(v) < best[0]):
            best = (int(v), m)
    return best
print('## 表 A：每条变异在合法历史域里最先被哪条判据打红（最小 n @ 模式），与对抗域')
print('| 变异 | R | P | M | X | F | 对抗域 R | 对抗域 P | 名字 |')
print('|---|---|---|---|---|---|---|---|---|')
tally = collections.Counter(); depth = collections.Counter()
for i in ids[1:]:
    got = {c: first(i, c, LEGAL) for c in 'RPMXF'}
    adv = {c: first(i, c, [ADV, ADV + '+n3']) for c in 'RP'}
    fmt = lambda b: '-' if b is None else '%d@%s' % b
    print('| %s | %s | %s | %s | %s | %s | %s | %s | %s |' % (i, *(fmt(got[c]) for c in 'RPMXF'), fmt(adv['R']), fmt(adv['P']), names[i][:60]))
    if got['R']: cls = '1 R（M2 字面，看根）就红'
    elif got['P']: cls = '2 要 P（发布的记录与点名单元都在）才红'
    elif got['M']: cls = '3 要 M（加在飞上限）才红'
    elif got['X'] or got['F']: cls = '4 只有 X / F（该施加的没施加、不动点）红：M1–M3 字面都不红'
    elif adv['R'] or adv['P']: cls = '5 合法历史域全绿，只在对抗域红'
    else: cls = '6 哪个域都不红'
    tally[cls] += 1
    ns = [b[0] for b in list(got.values()) + list(adv.values()) if b]
    if ns: depth[min(ns)] += 1
print('\n## 表 B：27 条变异按「最弱的能打红它的判据」分类')
for k in sorted(tally): print('%s\t%d' % (k, tally[k]))
print('合计\t%d' % sum(tally.values()))
print('\n## 表 C：能被打红的变异，最小杀伤界 n（记录条数）的分布')
for k in sorted(depth): print('n=%d\t%d 条' % (k, depth[k]))
print('\n## 表 D：原实现各模式的状态数与用时（campaign.log 里 orig 那一段）')
seen_orig = False
for line in open(sys.argv[2]):
    if line.startswith('=== '):
        if seen_orig: break
        seen_orig = True; continue
    m = re.match(r'(\S+)\tmode=\S+ n=(\d+) (?:shapes=\d+ )?(?:states|pairs)=(\d+) (.*)', line)
    if m:
        viol = ' '.join(re.findall(r'viol_\w+=\d+', m.group(4)))
        extra = ' '.join(re.findall(r'(?:errors|panics|secs)=[\d.]+', m.group(4)))
        print('%s\tn=%s\t%s\t%s\t%s' % (m.group(1), m.group(2), m.group(3), viol, extra))
