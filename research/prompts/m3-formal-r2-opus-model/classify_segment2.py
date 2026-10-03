"""E165 第二段每个变体：首红那个 N 上列出的判据里，去掉 R、U_verify_off、A、A6、AC 五个域之后还剩哪些。
参数：第二段产物路径（默认 research/results/e165-replay-bounded-enumeration-2026-10-01-segment2-kill-bounds.out）。"""
import re, sys
f = sys.argv[1] if len(sys.argv) > 1 else 'research/results/e165-replay-bounded-enumeration-2026-10-01-segment2-kill-bounds.out'
for line in open(f, encoding='utf-8'):
    if 'name=kill_bound' not in line or 'item=baseline' in line:
        continue
    item = re.search(r'item=(\S+)', line).group(1)
    row = re.search(r'table_row=(\S+)', line).group(1)
    fcr = re.search(r'first_clause_red=(\S+)', line).group(1)
    crit = set()
    if fcr != 'none':
        for part in fcr.split(';'):
            dom, n, cs = part.split(':', 2)
            if dom in ('R', 'U_verify_off', 'A', 'A6', 'AC'):
                continue
            for c in cs.split('+'):
                crit.add(c.split('[')[0])
    safety = crit & {'J0', 'J1', 'J2', 'J3'}
    cls = 'safety(J0-J3)' if safety else ('only J4/J5/J6' if crit else 'none in premise legal domains')
    print(item, row, cls, sorted(crit))
