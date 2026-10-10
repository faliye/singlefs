#!/usr/bin/env python3
"""汇总 OPUSI1-RUN 行：python3 agg.py <日志…> [--by key1,key2] [--show-checker N]"""
import re, sys, collections
args = [a for a in sys.argv[1:] if not a.startswith('--')]
by = 'sp,arm,cls,pattern,fault,refused,red,classes'
show = 0
for a in sys.argv[1:]:
    if a.startswith('--by='): by = a[5:]
    if a.startswith('--show-checker='): show = int(a[15:])
def parse(line):
    d = {}
    d['sp'] = re.search(r' sp=(\S+)', line).group(1)
    d['arm'] = re.search(r' arm="([^"]*)"', line).group(1) or 'prod'
    d['ops'] = re.search(r' ops=(\S+)', line).group(1)
    plan = re.search(r' plan=(\S+)', line).group(1)
    d['plan'] = plan
    m = re.match(r's1-(\w)@(\d+)/(\d+)\(d(\d)\)-trig(\d)-(\w)-(\w)', plan)
    if m:
        d['cls'], d['pattern'], d['fault'] = m.group(1), m.group(6), m.group(7)
    d['outcome'] = re.search(r' outcome=(\S+)', line).group(1)
    d['reached'] = re.search(r' reached=(\S+)', line).group(1)
    d['red'] = re.search(r' red=(\w+)', line).group(1)
    m = re.search(r'refused_at=(\d+):(\w+)', line)
    d['refused'] = f'{m.group(2)}@{m.group(1)}' if m else 'none'
    d['refused_kind'] = m.group(2) if m else 'none'
    m = re.search(r' classes=\{([^}]*)\}', line)
    d['classes'] = m.group(1).replace('"', '') if m else ''
    m = re.search(r' floors=\[([^\]]*)\]', line)
    d['floors'] = m.group(1) if m else ''
    d['ftop'] = d['floors'].split(', ')[-1] if d['floors'] else ''
    m = re.search(r'unmount=(\S+)', line)
    d['unmount'] = m.group(1) if m else '-'
    m = re.search(r'CHECKER (.*?)(?=", "|"\]$)', line)
    d['checker'] = m.group(1) if m else ''
    return d
rows = []
for f in args:
    for line in open(f, encoding='utf-8'):
        if line.startswith('OPUSI1-RUN'):
            rows.append(parse(line.rstrip('\n')))
keys = by.split(',')
c = collections.Counter(tuple(r.get(k, '') for k in keys) for r in rows)
print('OPUSI1-AGG by=' + by + f' rows={len(rows)}')
for k, n in sorted(c.items()):
    print('  ' + ' '.join(f'{a}={b}' for a, b in zip(keys, k)) + f' n={n}')
shown = 0
for r in rows:
    if show and r['checker'] and shown < show:
        print('OPUSI1-CHECKER', r['sp'], r['arm'], r['plan'], r['checker'][:1400].replace('\\"', '"'))
        shown += 1
