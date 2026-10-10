#!/usr/bin/env python3
"""汇总 c655probe y1 / y1s2 日志：每份日志的 MOUNT、REAL、I55 计数，字段判定按（动作, 关系, 字段, 判定）合计（r1 标签只留 queue_*）。
用法：summarize.py <日志…> [--fields 正则]"""
import re, sys, collections
args = [a for a in sys.argv[1:] if not a.startswith('--')]
pattern = re.compile(sys.argv[sys.argv.index('--fields') + 1]) if '--fields' in sys.argv else None
if pattern:
    args = [a for a in args if a != pattern.pattern]
for path in args:
    counts = collections.Counter()
    head = []
    for line in open(path, encoding='utf-8'):
        line = line.rstrip('\n')
        if line.startswith(('states_judged', 'second_crash_states_judged', 'first_crash_ordinal')):
            head.append(line)
        m = re.match(r'^(MOUNT|REAL|I55)(.*) count=(\d+)$', line)
        if m:
            counts[(m.group(1) + m.group(2))[:170]] += int(m.group(3))
            continue
        m = re.match(r'^FIELD action=(\S+) r1=\S+ (queue_\w+|no_entries) r1_rebuilt=\w+ (\S+) (\S+) = (.*) count=(\d+)$', line)
        if m:
            action, queue, relation, field, verdict, n = m.groups()
            if pattern and not pattern.search(field):
                continue
            counts[f'FIELD {action} {relation} {queue} {field} = {verdict}'] += int(n)
    print(f'== {path}')
    for h in head:
        print('  ' + h[:200])
    for k in sorted(counts):
        print(f'  {k} count={counts[k]}')
