#!/usr/bin/env python3
"""按 crates/mutations.tsv 的一行（行号）在草稿副本里复跑它：原文→替换文、跑第 5 列的 cargo 参数（只收 -p singlefs-harness 的，经 run-with-memory-cap.sh）、还原。
用法：run_row.py <行号> [上限]   输出：行名、cargo 退出码、结果行。"""
import os, re, subprocess, sys, shutil
S = '/tmp/claude-1000/m3-unlink-r2-forward'
FROZEN = '/tmp/claude-1000/m3-unlink-code-r2/tree'
row = int(sys.argv[1]); cap = sys.argv[2] if len(sys.argv) > 2 else '2G'
line = open(f'{FROZEN}/crates/mutations.tsv', encoding='utf-8').read().split('\n')[row - 1]
name, path, orig, repl, args, tests = line.split('\t')[:6]
un = lambda s: s.replace('\\n', '\n')
p = f'{S}/tree/{path}'
src = open(p, encoding='utf-8').read()
assert src.count(un(orig)) == 1, f'原文命中 {src.count(un(orig))} 次'
open(p, 'w', encoding='utf-8').write(src.replace(un(orig), un(repl)))
env = dict(os.environ, CARGO_TARGET_DIR=f'{S}/target',
           RUN_WITH_MEMORY_CAP_PARALLEL_REASON='正推腿 m3-unlink-r2-forward 与别的派出去的 agent 各跑各的，复跑变异行',
           RUN_WITH_MEMORY_CAP_WAIT_SECONDS='150',
           RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON='复跑变异行，上限给宽一点以免峰值表的两倍线误拒')
cmd = ['nice', '-n', '19', 'bash', '/home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh', cap,
       'bash', '/home/fy5090/code/singlefs/research/scripts/capped.sh', '6', 'cargo', 'test', '--offline'] + args.split()
log = f'{S}/row-{row}.log'
try:
    r = subprocess.run(cmd, cwd=f'{S}/tree', env=env, stdout=open(log, 'w'), stderr=subprocess.STDOUT, timeout=225)
    rc = r.returncode
except subprocess.TimeoutExpired:
    rc = 'timeout'
finally:
    shutil.copyfile(f'{FROZEN}/{path}', p)
print(f'row={row} name={name[:60]} rc={rc}')
for l in open(log, encoding='utf-8', errors='replace'):
    if l.startswith('test result') or ' FAILED' in l or 'panicked' in l:
        print('  ', l.rstrip()[:200])
