#!/usr/bin/env python3
"""汇总 T2 两臂：读 out/<阶段>.log 里 OPUSI1-RUN 行，按 (起点, S1 槽类, 回退结局, 期望, red) 数。用法：agg.py 日志…"""
import re, sys, collections
def kind(res):
    if res.startswith('Ok'): return 'Ok'
    m = re.match(r'Err\(txg\d+:(\w+)', res)
    return 'Err:' + (m.group(1) if m else res[:30])
rows = collections.Counter()
reds = collections.Counter()
checker = {}
for path in sys.argv[1:]:
    for line in open(path):
        if not line.startswith('OPUSI1-RUN'): continue
        sp = re.search(r' sp=(\S+)', line).group(1)
        plan = re.search(r' plan=(\S+)', line).group(1)
        cls = plan.split('@')[0]            # s1-V / s1-A
        outcome = re.search(r' outcome=(\S+)', line).group(1)
        mm = re.search(r' rollback=(\S+?)( expect=(\w+) trueF=(\d+))?(?= |$)', line)
        if not mm:
            note = 'none'; expect = '-'
        else:
            note = kind(mm.group(1)); expect = mm.group(3) or '-'
        red = re.search(r' red=(\w+)', line).group(1)
        sub = re.search(r'counters=\[(\d+),', line).group(1)
        key = (sp, cls, 'mount:'+outcome, note, 'expect='+expect, 'subst>0' if sub != '0' else 'subst=0', 'red='+red)
        rows[key] += 1
        if red == 'true' and key not in checker:
            f = re.search(r'findings=\[(.*)\]$', line)
            checker[key] = (f.group(1)[:700] if f else '')
for k, n in sorted(rows.items()):
    print(n, *k)
for k, v in checker.items():
    print('CHECKER-SAMPLE', *k, v)
