#!/usr/bin/env python3
"""gmut.py <json 文件> <编号…|all> : 对 tree-mut 的 gpu_memory_ledger.rs 套变异、编、跑 gpu_memory_ledger 库单测，打印失败的用例名。"""
import os, sys, json, subprocess, shutil, re
W = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-no-code-r1-sonnet')
FROZEN = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-no-code-r1/tree/crates/singlefs-harness/src/gpu_memory_ledger.rs')
DST = W + '/tree-mut/crates/singlefs-harness/src/gpu_memory_ledger.rs'
R = '/home/fy5090/code/singlefs/research/scripts'
rows = json.load(open(sys.argv[1]))
want = sys.argv[2:]
env = dict(os.environ, CARGO_TARGET_DIR=W + '/target-mut')
for i, r in enumerate(rows):
    if want != ['all'] and str(i) not in want and r.get('id') not in want:
        continue
    if not r.get('file', 'gpu_memory_ledger.rs').endswith('gpu_memory_ledger.rs'):
        continue
    src = open(FROZEN, encoding='utf-8').read()
    n = src.count(r['old'])
    if n != 1:
        print(i, r.get('id', ''), 'OLD-NOT-ONCE', n); continue
    open(DST, 'w', encoding='utf-8').write(src.replace(r['old'], r['new']))
    b = subprocess.run(['nice', '-n', '19', 'bash', R + '/run-with-memory-cap.sh', '12G', 'bash', R + '/capped.sh', '8', 'cargo', 'test', '--release', '--offline', '-p', 'singlefs-harness', '--lib', '--no-run'], cwd=W + '/tree-mut', env=env, capture_output=True, text=True)
    if b.returncode != 0:
        print(i, r.get('id', ''), 'BUILDFAIL', b.stderr[-300:].replace('\n', ' ')); continue
    exe = [l for l in b.stderr.split('\n') if 'unittests src/lib.rs' in l]
    path = re.search(r'\((.*)\)', exe[0]).group(1)
    t = subprocess.run(['nice', '-n', '19', 'bash', R + '/run-with-memory-cap.sh', '4G', 'bash', R + '/capped.sh', '4', path, 'gpu_memory_ledger'], cwd=W + '/tree-mut', env=env, capture_output=True, text=True)
    failed = [l.split(' ... ')[0].replace('test gpu_memory_ledger::tests::', '') for l in t.stdout.split('\n') if l.endswith('FAILED')]
    passed = sum(1 for l in t.stdout.split('\n') if l.endswith(' ok'))
    print(i, r.get('id', r.get('name', ''))[:90], '| rc', t.returncode, 'passed', passed, 'failed', len(failed))
    for f in failed: print('     FAILED:', f[:170])
    sys.stdout.flush()
shutil.copy(FROZEN, DST)
