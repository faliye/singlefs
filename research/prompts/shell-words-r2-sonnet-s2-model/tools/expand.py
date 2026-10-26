#!/usr/bin/env python3
"""expand.py <文本> : 把文本里的 {{名字}} 换成「原文」（`路径:行号`），{{sha}} 换成 sha 清单；名字在 frag.py 的 F 里，行号现 grep -nF。"""
import re, subprocess, sys, os
sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
from frag import F
REPO = '/home/fy5090/code/singlefs/'
PATHS = {'H': '.claude/hooks/heavy-test-guard.sh', 'B': '.claude/hooks/bash-command-detector.sh', 'L': '.claude/hooks/lib_heavy_tests.py',
         'W': '.claude/hooks/lib_shell_words.py', 'MA': '.claude/main-agent.md', 'AC': '.claude/agent-common.md'}
def cite(name):
    f, frag = F[name]
    r = subprocess.run(['grep', '-nF', '--', frag, REPO + PATHS[f]], capture_output=True, text=True)
    hits = [int(x.split(':', 1)[0]) for x in r.stdout.splitlines()]
    if not hits: raise SystemExit('找不到 ' + name)
    return f"「{re.sub(r'「[^」]*」', '…', frag)}」（`{PATHS[f]}:{hits[0]}`）"
text = open(sys.argv[1]).read()
out = re.sub(r'\{\{([a-z_0-9]+)\}\}', lambda m: cite(m.group(1)), text)
sys.stdout.write(out)
