#!/usr/bin/env python3
"""在草稿副本上套一行变异、跑点名的测试二进制（整个二进制，不按测试名滤）、记哪几条红、还原。
用法：mutrun.py <名字> <tsv行号|custom> [--orig F --repl F --file path --args "..."] [--filter-from-row]
行号取冻结副本 crates/mutations.tsv 的行。"""
import argparse, os, re, subprocess, sys, shlex, pathlib
S = pathlib.Path('/tmp/claude-1000/m3-batch10i-code-r1-sonnet')
WORK = S / 'work'
FROZEN_TSV = pathlib.Path('/tmp/claude-1000/m3-batch10i-code-r1/tree/crates/mutations.tsv')
R = '/home/fy5090/code/singlefs/scripts'
CAP = os.environ.get('MUT_CAP', 'auto')
ap = argparse.ArgumentParser()
ap.add_argument('name'); ap.add_argument('row')
ap.add_argument('--orig'); ap.add_argument('--repl'); ap.add_argument('--file'); ap.add_argument('--args')
ap.add_argument('--keep-filter', action='store_true', help='按表里第 5 段原样跑（含测试名过滤）')
a = ap.parse_args()
unesc = lambda s: s.replace('\\n', '\n')
if a.row != 'custom':
    cols = FROZEN_TSV.read_text().split('\n')[int(a.row) - 1].split('\t')
    file, orig, repl, args = cols[1], unesc(cols[2]), unesc(cols[3]), cols[4]
else:
    file, orig, repl, args = a.file, pathlib.Path(a.orig).read_text(), pathlib.Path(a.repl).read_text(), a.args
if a.args: args = a.args
path = WORK / file
text = path.read_text()
n = text.count(orig)
if n != 1:
    print(f'锚点命中 {n} 次，不是 1'); sys.exit(3)
cmd = f'cargo test --offline {args}'
if not a.keep_filter and ' -- ' not in args:
    pass
log = S / f'mut-{a.name}.log'
env = dict(os.environ, CARGO_TARGET_DIR=str(S / 'target'))
path.write_text(text.replace(orig, repl))
try:
    r = subprocess.run(['nice', '-n', '19', 'bash', f'{R}/run-with-memory-cap.sh', CAP, 'bash', f'{R}/capped.sh', '15'] + shlex.split(cmd),
                       cwd=WORK, env=env, capture_output=True, text=True, timeout=230)
    out = r.stdout + r.stderr
    code = r.returncode
except subprocess.TimeoutExpired as e:
    out = (e.stdout or b'').decode() if isinstance(e.stdout, bytes) else (e.stdout or ''); code = 'timeout'
finally:
    path.write_text(text)
log.write_text(out)
failed = re.findall(r'^test (\S+) \.\.\. FAILED', out, re.M)
okc = len(re.findall(r'^test (\S+) \.\.\. ok', out, re.M))
res = re.findall(r'^test result: .*$', out, re.M)
print(f'[{a.name}] exit={code} ok={okc} failed={len(failed)}')
for f in failed: print('  FAILED', f)
for l in res: print(' ', l)
