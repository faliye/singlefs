#!/usr/bin/env python3
"""汇总 opus_10y_k4_probe 的 OPUS10Y-K4 行：按 (盘宽, 臂, C 是不是 S1, 卸载前 C 那一槽被写掉没有) 数卸载结局与后续。
用法：python3 k4agg.py <日志…>"""
import re, sys, collections

def field(line, name):
    m = re.search(r'(?:^| )' + re.escape(name) + r'=(\{[^}]*\}|\[[^\]]*\]|"[^"]*"|\S+)', line)
    return m.group(1) if m else '-'

rows = collections.Counter()
for path in sys.argv[1:]:
    for line in open(path, encoding='utf-8'):
        i = line.find('OPUS10Y-K4 sp=')
        if i < 0:
            continue
        line = line[i:].rstrip('\n')
        short = lambda v: v.replace('NewerStateStillUnreadableAfterOneReread/', '')
        width = field(line, 'sp').split('-')[1]
        arm = field(line, 'arm').strip('"') or 'prod'
        s1 = 'plain' if field(line, 's1') == 'None' else 'S1'
        refusals = field(line, 'write_refusals')
        key = (width, arm, s1,
               'C-left' if field(line, 'c_left_before_unmount') == 'true' else 'C-in-ring',
               'unmount=' + short(field(line, 'unmount')),
               'after_unmount_I74=' + field(line, 'after_unmount_I-7.4'),
               'writes_after_refusal=' + ('-' if refusals == '-' else ('ok' if refusals == '{}' else 'refused')),
               'second_unmount=' + short(field(line, 'second_unmount')),
               'remount_with_fault=' + short(field(line, 'remount_with_fault')),
               'worst_I74_after_clean_remount=' + field(line, 'worst_I-7.4_after'))
        rows[key] += 1
for key, n in sorted(rows.items()):
    print(n, ' '.join(key))
print('合计', sum(rows.values()))
