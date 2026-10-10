"""把 out/ 下各步的输出归成与跑的快慢、目录无关的摘要行（去掉耗时、编译目录、哈希后缀），按文件名排序。"""
import hashlib
import os
import re
import subprocess
import sys

out = sys.argv[1]
here = os.path.dirname(os.path.abspath(__file__))
lines = []

def norm_test_line(line):
    line = re.sub(r'; finished in [0-9.]+s', '', line)
    return line.rstrip()

for name in sorted(os.listdir(out)):
    if not name.endswith('.out') or name.startswith('summary'):
        continue
    path = os.path.join(out, name)
    text = open(path, encoding='utf-8', errors='replace').read().splitlines()
    tag = name[:-4]
    if tag.startswith('n5-probe-'):
        result = subprocess.run([sys.executable, os.path.join(here, 'n5summary.py'), path], capture_output=True, text=True, check=True)
        lines += [f'{tag} {row}' for row in result.stdout.splitlines()]
        continue
    if tag == 'sweep':
        history = [row for row in text if row.startswith('SWEEP_HISTORY ')]
        lines.append(f'{tag} SWEEP_HISTORY_LINES count={len(history)} sha256_16={hashlib.sha256(chr(10).join(history).encode()).hexdigest()[:16]}')
        lines += [f'{tag} {row}' for row in text if row.startswith('SWEEP_TOTALS')]
        flagged = [row for row in history if 'flagged=[' in row]
        lines.append(f'{tag} SWEEP_FLAGGED_HISTORIES {len(flagged)}')
        continue
    for row in text:
        if row.startswith('     Running '):
            lines.append(f'{tag} ' + re.sub(r' \(.*\)$', '', row.strip()))
        elif re.match(r'^test .* \.\.\. (ok|FAILED|ignored)$', row) or row.startswith('test result:'):
            lines.append(f'{tag} ' + norm_test_line(row))
        elif row.startswith(('N7_PROBE', 'G5 ', 'SHAPE ', 'BADIMAGE ', 'CRASH_COUNT', 'exit=')) or row.strip().startswith(('left:', 'right:')):
            lines.append(f'{tag} ' + row.strip())
print('\n'.join(lines))
