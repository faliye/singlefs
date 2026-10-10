#!/usr/bin/env python3
"""agg_table.py <日志…>：table-* 动作的行按 (动作, 要出路头, 真推环, 真再发是否盘表那种, 真放得下, 要出路读盘数干净) 计数"""
import re, sys, collections
c = collections.Counter()
for f in sys.argv[1:]:
    for line in open(f, errors='replace'):
        m = re.search(r'act=(table-[a-z]+): (.*)', line)
        if not m: continue
        act, rest = m.groups()
        mh = re.match(r'ask=(\w+)', rest)
        head = mh.group(1) if mh else 'noask:' + rest[:30]
        if 'truth_push=' in rest:
            tp = re.search(r'truth_push=(\w+)', rest).group(1)
            rd = re.search(r'retry_is_devtable=(\w+)', rest).group(1)
            ta = re.search(r'truth_admitted=(\w+)', rest).group(1)
            io = re.search(r'io_clean=(\w+)', rest).group(1)
            mm = 'MISMATCH' if 'MISMATCH' in rest else 'ok'
            c[(act, head, f'truth_push={tp}', f'retry_is_devtable={rd}', f'truth_admitted={ta}', f'io_clean={io}', mm)] += 1
        else:
            ag = re.search(r'agree=(\w+)', rest)
            c[(act, head, 'rehearsal-compare', f'agree={ag.group(1) if ag else "?"}', 'MISMATCH' if 'MISMATCH' in rest else 'ok')] += 1
for k, v in sorted(c.items()): print(v, *k)
