#!/usr/bin/env python3
"""T1：真仓实验页标题的状态段，按快照树 75 号的两个判法各归一类（只读）。
  ran_status：「——」之后、第一个全角括号之前，去空白后恰好是「已跑」（75 号第 386–389 行）。
  voided：标题整行里出现「作废」或「退役」（75 号第 287 行），⑨ 不判这些页。
另列 40 号认的「已跑|已测」（40 号第 53 行）与 ran_status 不一致的页。
用法：python3 t1-titles.py <仓根>
"""
import collections, glob, os, re, sys
os.chdir(sys.argv[1])
statuses, voided_rows, disagree = collections.Counter(), [], []
for path in sorted(glob.glob('.claude/kb/experiments/*.md')):
    title = next((l.rstrip('\n') for l in open(path, encoding='utf-8') if re.match(r'^## E\d+ ', l)), None)
    if not title:
        continue
    number = re.match(r'^## (E\d+) ', title).group(1)
    m = re.match(r'^## E\d+ .*?——\s*(.*)$', title)
    status = re.sub(r'（.*$', '', m.group(1)).strip() if m else ''
    statuses[status] += 1
    ran = status == '已跑'
    if re.search(r'作废|退役', title):
        where = [title[max(0, x.start() - 10):x.end()] for x in re.finditer(r'作废|退役', title)]
        voided_rows.append((number, status, where))
    if bool(re.search(r'已跑|已测', title)) and not ran and '作废' not in status and not status.startswith('部分'):
        disagree.append((number, status))
print(f'实验页 {sum(statuses.values())} 份；状态段恰好是「已跑」的 {statuses["已跑"]} 份')
print(f'40 号认作跑过（标题里有 已跑|已测）、75 号 ⑤ 后一半不认作已跑、又不是「部分…」「…作废」的 {len(disagree)} 份：')
for number, status in disagree:
    print(f'  {number}：「{status[:30]}」')
print(f'标题里出现「作废」或「退役」、⑨ 因此不判的 {len(voided_rows)} 份：')
for number, status, where in voided_rows:
    print(f'  {number}：状态段「{status[:16]}」；命中 {where}')
