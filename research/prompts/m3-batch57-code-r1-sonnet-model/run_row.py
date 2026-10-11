#!/usr/bin/env python3
# 用法：run_row.py <mutations.tsv 里的行号…>：把那一行的原文换成替换文（原文须在冻结副本文件里恰好命中一次），
# 在我的拷贝里跑那一行的 cargo test 参数，看必须红的测试名是不是出现在 FAILED 里；跑完把文件还原成冻结副本的样子。
import os, subprocess, sys, shlex
W = os.path.expanduser('~/.local/share/singlefs/work/m3-batch57-code-r1-sonnet')
FROZEN = os.path.expanduser('~/.local/share/singlefs/work/m3-batch57-code-r1/repo')
rows = open(f'{FROZEN}/crates/mutations.tsv', encoding='utf-8').read().split('\n')
env = dict(os.environ, CARGO_TARGET_DIR=f'{W}/target')
for ln in map(int, sys.argv[1:]):
    f = rows[ln-1].split('\t')
    name, file, old, new, args, red = f[0], f[1], f[2], f[3], f[4], f[5]
    old = old.replace('\\n', '\n'); new = new.replace('\\n', '\n')
    src = open(f'{FROZEN}/{file}', encoding='utf-8').read()
    n = src.count(old)
    if n != 1:
        print(f'ROW {ln} 锚点命中 {n} 次，跳过'); continue
    open(f'{W}/repo/{file}', 'w', encoding='utf-8').write(src.replace(old, new))
    cmd = ['nice', '-n', '19', 'bash', '/home/fy5090/code/singlefs/scripts/hosts/run-with-memory-cap.sh', '10G',
           'bash', '/home/fy5090/code/singlefs/scripts/hosts/capped.sh', '10', 'cargo', 'test', '--offline'] + shlex.split(args)
    p = subprocess.run(cmd, cwd=f'{W}/repo', env=env, capture_output=True, text=True)
    open(f'{W}/row-{ln}.log', 'w').write(p.stdout + p.stderr)
    open(f'{W}/repo/{file}', 'w', encoding='utf-8').write(src)
    failed = [l for l in (p.stdout).split('\n') if l.startswith('test ') and l.endswith('FAILED')]
    hit = any(red in l for l in failed)
    print(f'ROW {ln} rc={p.returncode} 必须红的测试{"红了" if hit else "没红"}；FAILED 共 {len(failed)} 条；{name[:60]}')
