#!/usr/bin/env python3
"""rerun.sh 的 rerun_steps 调它：在 <草稿目录>/out 里把模型重跑一遍，与 <模型目录>/out 里录下的逐字节比（去掉 wall_ms 行）。
用法：rerun_steps.py <模型目录> <草稿目录>"""
import os
import subprocess
import sys

model, work = sys.argv[1], sys.argv[2]
out_dir = os.path.join(work, 'out')
os.makedirs(out_dir, exist_ok=True)
env = dict(os.environ, PYTHONDONTWRITEBYTECODE='1')
SNAP = os.path.join(model, 'snapmodel.py')
TRACE = os.path.join(model, 'trace.py')
NICE = ['nice', '-n', '19', 'python3']

INDEP_GROUPS = [
    'base-indep,a1-jia-stxg-from-birth,a1c-jia-stxg-from-birth-prev-creation,a2-jia-stxg-from-chain',
    'a4-meta-judged,a5-node-release-control,a6-flag-explicit,a7-flag-jia-birth',
    'a7c-flag-jia-chain,a9-rb-release-live,a10-rb-resurrect-live,a11-rb-today',
    'a11r-rb-today-refuse-diff,a11d-rb-destroying-included,a12-pass-final-control,a13-flag-no-interleave',
    'a8i-indep-tie-reverse,a14-rb-today-split-yi,a14r-rb-today-split-jia,a14a-rb-split-only',
]
SAME_GROUPS = ['base-same', 'a2s-jia-stxg-from-chain-same', 'a3-same-killprev-new', 'a8-same-tie-reverse']
RANDOM_GROUPS = ['base-indep', 'base-same', 'a1c-jia-stxg-from-birth-prev-creation', 'a2-jia-stxg-from-chain',
                 'a2s-jia-stxg-from-chain-same', 'a6-flag-explicit', 'a13-flag-no-interleave']


def run_into(name, commands):
    with open(os.path.join(out_dir, name), 'w') as out:
        for cmd in commands:
            if cmd[0] == '===':
                out.write('=== ' + ' '.join(cmd[1:]) + '\n')
                out.flush()
                cmd = NICE + [TRACE] + cmd[1:]
            subprocess.run(cmd, stdout=out, check=True, env=env)


run_into('depth6-indep.jsonl', [NICE + [SNAP, '--depth', '6', '--only', g] for g in INDEP_GROUPS])
run_into('depth5-same.jsonl', [NICE + [SNAP, '--depth', '5', '--only', g] for g in SAME_GROUPS])
run_into('random16.jsonl', [NICE + [SNAP, '--random', '200000', '--length', '16', '--seed', '7', '--only', g]
                            for g in RANDOM_GROUPS])
with open(os.path.join(model, 'out', 'traces.list')) as f:
    run_into('traces.txt', [['==='] + line.split() for line in f if line.strip()])

bad = 0
for name in ['depth6-indep.jsonl', 'depth5-same.jsonl', 'random16.jsonl', 'traces.txt']:
    with open(os.path.join(model, 'out', name)) as f:
        recorded = [line for line in f if not line.startswith('wall_ms=')]
    with open(os.path.join(out_dir, name)) as f:
        fresh = f.readlines()
    same = recorded == fresh
    bad += 0 if same else 1
    print('name=rerun_compare file=%s identical=%s lines=%d' % (name, 'true' if same else 'false', len(fresh)))
sys.exit(1 if bad else 0)
