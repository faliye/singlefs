#!/usr/bin/env python3
"""replay_scan.py <gpu|crash> <行号…> : 把改动全文里第二十一块变异行的 旧串→新串 套到扫源码模型（replay/<kind>_model.rs），rustc 编、跑，报哪些用例红。"""
import os, re, subprocess, sys
W = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-tu-code-r1-sonnet')
R = '/home/fy5090/code/singlefs/scripts'
S = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-tu-code-r1/tree/crates/singlefs-checker-tier/src/')
kind = sys.argv[1]
L = open('/home/fy5090/code/singlefs/research/prompts/_m3-batch10z-tu-code-r1-diff.md').read().split('\n')
model = open(f'{W}/replay/{kind}_model.rs').read()
srcfile = S + ('gpu_unit_checks.rs' if kind == 'gpu' else 'crash_amplification.rs')
envname = 'GPU_SRC' if kind == 'gpu' else 'CRASH_SRC'
for n in map(int, sys.argv[2:]):
    f = L[n - 1].split('\t')
    name, old, new = f[0][-60:], f[2].replace('\\n', '\n'), f[3].replace('\\n', '\n')
    target_is_product = f[1].endswith('crash_amplification.rs') and ('judge_my_blocks_on_gpu' in f[0] and '不问叫停' in f[0])
    if target_is_product:
        text = open(srcfile).read()
        if text.count(old) != 1:
            print(f'行{n}: 产品源码旧串命中 {text.count(old)} 次'); continue
        mutated_src = f'{W}/replay/{kind}_mutated_src.rs'
        open(mutated_src, 'w').write(text.replace(old, new, 1))
        out_bin = f'{W}/replay/{kind}_row{n}'
        subprocess.run(['rustc', '--edition', '2021', '-O', '-o', out_bin, f'{W}/replay/{kind}_model.rs'], capture_output=True)
        r = subprocess.run(['bash', R + '/run-with-memory-cap.sh', '1G', out_bin], capture_output=True, text=True, env=dict(os.environ, **{envname: mutated_src}))
    else:
        if model.count(old) != 1:
            print(f'行{n}: 模型里旧串命中 {model.count(old)} 次 ({name})'); continue
        mp = f'{W}/replay/{kind}_row{n}.rs'
        open(mp, 'w').write(model.replace(old, new, 1))
        out_bin = f'{W}/replay/{kind}_row{n}'
        c = subprocess.run(['rustc', '--edition', '2021', '-O', '-o', out_bin, mp], capture_output=True, text=True)
        if c.returncode != 0:
            print(f'行{n}: 编不过 {[l for l in c.stderr.splitlines() if l.startswith("error")][:1]}'); continue
        r = subprocess.run(['bash', R + '/run-with-memory-cap.sh', '1G', out_bin], capture_output=True, text=True, env=dict(os.environ, **{envname: srcfile}))
    fails = re.findall(r'^FAIL (\S+)', r.stdout, re.M)
    passes = len(re.findall(r'^PASS', r.stdout, re.M))
    print(f'行{n} {name}: 红 {len(fails)} 条 {[x[:70] for x in fails]} 绿 {passes}')
