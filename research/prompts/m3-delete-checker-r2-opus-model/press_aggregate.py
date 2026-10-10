#!/usr/bin/env python3
"""out/bad-press.out 按压之后再发几次（follow）分组：每个候选在「号被重发」的那几行里红了几行；今天的 checker 一条都不红的行按「哪几个候选红」分组。行首 PRESSAGG。"""
import collections, re, sys

names = ["T2a", "T2c-A", "T2c-A2", "T2a/selmap", "yi/rec", "yi/rec+tomb", "bing", "I96code", "I96newest", "I96each", "I919A"]
pattern = re.compile(r'prefix=(\S+) wm_before=(\d+) p=(\d+) n=(\d+) follow=(\w+) reissue=(\w+) outcome=\[([^\]]*)\] marks=(\S+) checker=(\[[^\]]*\])')
rows = []
for line in open(sys.argv[1], encoding='utf-8'):
    if line.startswith('PRESS '):
        rows.append(pattern.search(line).groups())
by_follow = collections.defaultdict(list)
for row in rows:
    by_follow[int(row[4])].append(row)
for follow, group in sorted(by_follow.items()):
    reissued = [row for row in group if row[5] == 'true']
    print(f'PRESSAGG follow={follow} rows={len(group)} reissue={len(reissued)} checker_any_red={sum(1 for row in reissued if row[8] != "[]")}')
    print('PRESSAGG follow=%d caught ' % follow + ' '.join(f'{name}={sum(1 for row in reissued if row[7][index] == "R")}' for index, name in enumerate(names)))
    silent = collections.Counter(tuple(name for index, name in enumerate(names) if row[7][index] == 'R') for row in reissued if row[8] == '[]')
    for catching, count in sorted(silent.items(), key=lambda item: (-item[1], item[0])):
        print(f'PRESSAGG follow={follow} checker_silent rows={count} caught_by={list(catching)}')
    for row in reissued:
        if row[8] == '[]' and row[7][0] == 'R' and row[7][1] != 'R' and row[7][5] != 'R':
            print(f'PRESSAGG follow={follow} only_T2a_family prefix={row[0]} p={row[2]} n={row[3]} marks={row[7]}')
