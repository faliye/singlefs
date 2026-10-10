#!/usr/bin/env python3
"""replay23.py <行文件.tsv> [行号…] ：把第二十三块变异行的 旧串→新串 套到扫源码小程序（build/{gpu,crash}-main.rs），rustc 编、跑，报哪些用例红、点名的那条红没红。"""
import os, re, subprocess, sys
W = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-vw-code-r1-sonnet/scan')
R = '/home/fy5090/code/singlefs/research/scripts'
S = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-vw-code-r1/tree/crates/singlefs-checker-tier/src/')
rows = [l.rstrip('\n').split('\t') for l in open(sys.argv[1]) if l.strip()]
only = set(map(int, sys.argv[2:])) or set(range(1, len(rows) + 1))
for n, f in enumerate(rows, 1):
    if n not in only: continue
    name, path, old, new, args, test = f[:6]
    kind = 'gpu' if path.endswith('gpu_unit_checks.rs') else 'crash'
    src = 'gpu_unit_checks.rs' if kind == 'gpu' else 'crash_amplification.rs'
    model = open(f'{W}/build/{kind}-main.rs').read()
    o, nw = old.replace('\\n', '\n'), new.replace('\\n', '\n')
    # 产品源码里的行（不在小程序里）：套在被扫的源码上
    if model.count(o) != 1:
        text = open(S + src).read()
        if text.count(o) == 1:
            mutated = f'{W}/build/row{n}_src.rs'
            open(mutated, 'w').write(text.replace(o, nw, 1))
            env = dict(os.environ, SRC=mutated); binary = f'{W}/bin/{kind}'
            r = subprocess.run(['bash', R + '/run-with-memory-cap.sh', '2G', binary], capture_output=True, text=True, env=env)
        else:
            print(f'行{n} {name[-50:]}: 小程序里旧串命中 {model.count(o)} 次、源码里 {text.count(o)} 次'); continue
    else:
        mp = f'{W}/build/row{n}.rs'
        open(mp, 'w').write(model.replace(o, nw, 1))
        out = f'{W}/bin/row{n}'
        c = subprocess.run(['rustc', '--edition', '2021', '-O', '-o', out, mp], capture_output=True, text=True)
        if c.returncode != 0:
            print(f'行{n} {name[-50:]}: 编不过 {[l for l in c.stderr.splitlines() if l.startswith("error")][:1]}'); continue
        r = subprocess.run(['bash', R + '/run-with-memory-cap.sh', '2G', out], capture_output=True, text=True, env=dict(os.environ, SRC=S + src))
    fails = re.findall(r'^(\S+): RED', r.stdout, re.M)
    passes = len(re.findall(r': GREEN', r.stdout))
    hit = any(x.endswith(test) or test.endswith(x) or x == test for x in fails)
    print(f'行{n} {name[-44:]}: 点名的{"红" if hit else "没红"}；红 {len(fails)} 条，绿 {passes}；点名 {test[:60]}')
