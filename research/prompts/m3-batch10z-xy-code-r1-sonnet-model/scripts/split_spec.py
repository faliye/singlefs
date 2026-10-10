#!/usr/bin/env python3
"""split_spec.py <规格文件> [完成的名字…]：按 S2_PREFIXES 的分号拆成每件一个前缀；已有 .rc 的名字（runs/<名>.rc）跳过，输出到标准输出"""
import sys, re, os, shlex
W = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-xy-code-r1-sonnet')
for line in open(sys.argv[1]):
    line = line.strip()
    if not line: continue
    parts = shlex.split(line)
    name = parts[0]; env = parts[1:]
    if os.path.exists(f'{W}/runs/{name}.rc') and open(f'{W}/runs/{name}.rc').read().strip() == '0': continue
    kv = [e.split('=', 1) for e in env]
    pre = [v for k, v in kv if k == 'S2_PREFIXES']
    prefixes = pre[0].split(';') if pre else ['']
    for i, p in enumerate(prefixes):
        n2 = f'{name}_p{i}' if len(prefixes) > 1 else name
        if os.path.exists(f'{W}/runs/{n2}.rc') and open(f'{W}/runs/{n2}.rc').read().strip() == '0': continue
        e2 = [f'{k}={shlex.quote(v)}' if k != 'S2_PREFIXES' else f'S2_PREFIXES={shlex.quote(p)}' for k, v in kv]
        e2 = [x.replace(f'S2_LABEL={name}', f'S2_LABEL={n2}') if x.startswith('S2_LABEL=') else x for x in e2]
        print(n2, ' '.join(e2))
