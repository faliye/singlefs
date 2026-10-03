#!/usr/bin/env python3
"""汇总 window_fuzz 的输出：每个文件的段数、候选类命中段、被抛弃类命中段、checker 红的段（按不变量计）、挂载被拒中止的段。"""
import re, sys, collections
for path in sys.argv[1:]:
    seeds = cand = aband = red = aborted = 0
    inv = collections.Counter()
    for line in open(path):
        if not line.startswith('FUZZ '):
            continue
        seeds += 1
        if not re.search(r' candidate_hits=0 ', line): cand += 1
        if not re.search(r' abandoned_hits=0 ', line): aband += 1
        m = re.search(r'checker_reds=\[(.*?)\] refusals=', line)
        if m and m.group(1):
            red += 1
            for name in set(re.findall(r'I-[0-9.]+', m.group(1))): inv[name] += 1
        if 'aborted=""' not in line: aborted += 1
    print(f"{path} seeds={seeds} cand_seeds={cand} aband_seeds={aband} checker_red_seeds={red} by_invariant={dict(sorted(inv.items()))} aborted={aborted}")
