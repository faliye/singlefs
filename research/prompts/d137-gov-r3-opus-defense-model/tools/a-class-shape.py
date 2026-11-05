#!/usr/bin/env python3
"""A 类（cd 目标盘上不在）新拒里，被起的脚本是同一条命令里 cd 之后用 cat 读 heredoc 写出的有几条：
a-class-shape.py <d137-replay-classify/an/real_new.json>。正则粗分（推的），不逐条展开核。
cd 后面接 `;` 或换行、cd 与写出之间没有 && / || 的，写出与执行落在同一个目录（不管 cd 成不成），记 cd-then-semicolon-or-newline。"""
import json, re, collections, sys
D = json.load(open(sys.argv[1]))
c = collections.Counter(); rows = []
for o in D:
    if not o['cls'].startswith('A'): continue
    m = re.search(r'起的脚本 (\S+?) 要先 cd 到 (\S+?)，', o['why'][0])
    if not m: c['no-script-name'] += 1; continue
    s, d = m.group(1), m.group(2); dbase = d.rstrip('/').split('/')[-1]; cmd = o['cmd']
    hd = re.search(r'cat\s*>\s*"?' + re.escape(s) + r'"?\s*<<', cmd)
    if not hd: c['script-not-heredoc-written'] += 1; continue
    cdm = [x for x in re.finditer(r'(^|[;&|\n(]\s*)cd\s+([^;&|\n]*)(;|&&|\n|\|\|)', cmd) if dbase in x.group(2) and x.start() < hd.start()]
    if not cdm: c['no-cd-match'] += 1; continue
    sep = cdm[-1].group(3); between = cmd[cdm[-1].end():hd.start()]
    k = 'cd-then-semicolon-or-newline' if sep in (';', '\n') and '&&' not in between.split('\n')[-1] and '||' not in between else 'cd-then-and-or-chain'
    c[k] += 1
    if k.startswith('cd-then-semi'): rows.append(o['line'])
print('name=a-class-heredoc-shape total_a=%d %s' % (sum(c.values()), ' '.join(f'{k}={v}' for k, v in sorted(c.items()))))
print('name=a-class-heredoc-shape-lines lines=' + ','.join(map(str, rows)))
