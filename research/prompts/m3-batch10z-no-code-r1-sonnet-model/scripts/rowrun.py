#!/usr/bin/env python3
"""rowrun.py <json> <编号…>: 对 tree-mut 套实现员的变异行（只动 allocator.rs / mount.rs），按行里的命令编并跑点名的用例。"""
import os, sys, json, subprocess, shutil
W = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-no-code-r1-sonnet')
R = '/home/fy5090/code/singlefs/scripts'
rows = json.load(open(sys.argv[1]))
env = dict(os.environ, CARGO_TARGET_DIR=W + '/target-mut')
for r in rows:
    if r['id'] not in sys.argv[2:]:
        continue
    for f in ('allocator.rs', 'mount.rs'):
        shutil.copy(W + '/tree-core/crates/singlefs-core/src/' + f, W + '/tree-mut/crates/singlefs-core/src/' + f)
    path = W + '/tree-mut/' + r['file']
    s = open(path, encoding='utf-8').read()
    if s.count(r['old']) != 1:
        print(r['id'], 'OLD-NOT-ONCE', s.count(r['old'])); continue
    open(path, 'w', encoding='utf-8').write(s.replace(r['old'], r['new']))
    args = r['cmd'].split()
    cmd = ['nice', '-n', '19', 'bash', R + '/run-with-memory-cap.sh', '8G', 'bash', R + '/capped.sh', '4', 'cargo', 'test', '--release', '--offline'] + args
    p = subprocess.run(cmd, cwd=W + '/tree-mut', env=env, capture_output=True, text=True)
    out = p.stdout
    ok = out.count(' ... ok'); bad = [l for l in out.split('\n') if l.endswith('FAILED')]
    print(r['id'], r['name'][:70], '| rc', p.returncode, '| ok', ok, 'failed', len(bad))
    for b in bad: print('     FAILED:', b.replace('test ', '')[:170])
    if p.returncode not in (0, 101): print(p.stderr[-300:])
    sys.stdout.flush()
for f in ('allocator.rs', 'mount.rs'):
    shutil.copy(W + '/tree-core/crates/singlefs-core/src/' + f, W + '/tree-mut/crates/singlefs-core/src/' + f)
