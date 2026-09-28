#!/usr/bin/env python3
"""cellview.py <日志> <正则>：命中的 AF-CELL 行压成一行：候选、键、抬 F 结局与抬后违例、各段后缀的违例链（只留判定，去掉挂载摘要）。"""
import re, sys
verdict = re.compile(r'=>(\[[^\]]*\])')
for line in open(sys.argv[1], errors='replace'):
    if 'AF-CELL' not in line or not re.search(sys.argv[2], line):
        continue
    head = line.split('AF-CELL ', 1)[1]
    key = head.split(' raise=')[0].split(' post=')[0]
    raise_part = head.split(' raise=', 1)[1].split(' post=', 1)[0] if ' raise=' in head else '-'
    raise_short = re.sub(r'\[requested.*?\}\]', '', raise_part)
    posts = []
    if ' post=[' in head:
        for seg in head.split(' post=[', 1)[1].rstrip().rstrip(']').split(' | '):
            if ':' in seg:
                name, body = seg.split(':', 1)
                chain = '>'.join(verdict.findall(body)) or body.split('=>')[0][:40]
                posts.append(f"{name}:{chain}")
    print(f"{key} raise={raise_short} post=[{' | '.join(posts)}]")
