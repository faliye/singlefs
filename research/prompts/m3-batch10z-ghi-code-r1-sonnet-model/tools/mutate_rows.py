#!/usr/bin/env python3
"""Apply each harness-level mutation row from the diff file on the copy, run its test, restore. Own tool."""
import subprocess, sys, os, re, shutil
DIFF = '/home/fy5090/code/singlefs/research/prompts/_m3-batch10z-ghi-code-r1-diff.md'
COPY = sys.argv[1]
TARGET = sys.argv[2]
SEL = sys.argv[3] if len(sys.argv) > 3 else None   # substring filter on name
rows = []
in_fence = False
for line in open(DIFF, encoding='utf-8'):
    line = line.rstrip('\n')
    if line.startswith('````') and not line.startswith('````diff'):
        in_fence = not in_fence
        continue
    if in_fence and line.count('\t') == 5 and line.startswith('第10z批'):
        rows.append(line.split('\t'))
sel = [r for r in rows if r[4].startswith('-p singlefs-harness') or r[4].startswith('-p singlefs-core')]
if SEL:
    sel = [r for r in sel if SEL in r[0]]
print('rows', len(rows), 'selected', len(sel), flush=True)
for name, path, old, new, cmd, test in sel:
    old = old.replace('\\n', '\n'); new = new.replace('\\n', '\n')
    f = os.path.join(COPY, path)
    src = open(f, encoding='utf-8').read()
    n = src.count(old)
    if n != 1:
        print('SKIP', name[:60], 'old occurs', n, flush=True); continue
    open(f, 'w', encoding='utf-8').write(src.replace(old, new))
    try:
        args = ['bash', '/home/fy5090/code/singlefs/scripts/hosts/capped.sh', '15', 'bash',
                '/home/fy5090/code/singlefs/scripts/hosts/run-with-memory-cap.sh', '8G', 'nice', '-n', '19',
                'cargo', 'test', '--offline'] + cmd.split()
        env = dict(os.environ, CARGO_TARGET_DIR=TARGET)
        p = subprocess.run(args, cwd=COPY, env=env, capture_output=True, text=True, timeout=1500)
        out = p.stdout + p.stderr
        m = re.findall(r'test result: .*', out)
        verdict = 'RED(caught)' if p.returncode != 0 else 'GREEN(not caught)'
        print(verdict, '|', name[:70], '| rc', p.returncode, '|', (m[-1] if m else out[-200:].replace('\n',' ')), flush=True)
    finally:
        open(f, 'w', encoding='utf-8').write(src)
