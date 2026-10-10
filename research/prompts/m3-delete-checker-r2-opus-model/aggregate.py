#!/usr/bin/env python3
"""合法历史各家族（out/sweep-*.out 的 COUNT 行）按候选合计：成立 / 红 / 无对象的状态数；CHECKERRED 按不变量合计。行首 AGG。"""
import collections, glob, os, sys

out = sys.argv[1]
totals = collections.defaultdict(lambda: [0, 0, 0])
checker = collections.Counter()
states = 0
for path in sorted(glob.glob(os.path.join(out, 'sweep-*.out'))):
    for line in open(path, encoding='utf-8'):
        parts = line.rstrip('\n').split('\t')
        if line.startswith('COUNT'):
            name = parts[2]
            for index, field in enumerate(parts[3:6]):
                totals[name][index] += int(field.split('=')[1])
        elif line.startswith('CHECKERRED'):
            checker[parts[2]] += int(parts[3])
        elif line.startswith('SUMMARY'):
            states += int(line.split('states=')[1])
print(f'AGG states={states}')
for name, (holds, red, na) in sorted(totals.items()):
    print(f'AGG {name} holds={holds} red={red} na={na}')
for name, count in sorted(checker.items()):
    print(f'AGG checker_red {name} states={count}')
