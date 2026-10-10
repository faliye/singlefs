#!/usr/bin/env python3
"""depth_check.py：把 logs/final/depth.out（扫源码模型对 src/depth-cases.txt 每个 case 算出的每行起头层数）对 src/depth-expect.json 里我手算的期望逐个比。"""
import json, os
root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
exp = json.load(open(root + '/src/depth-expect.json'))
out = open(root + '/logs/final/depth.out').read().splitlines()
bad = 0
for i, ((desc, e), o) in enumerate(zip(exp, out)):
    got = o.split(': ', 1)[1]
    ok = got == str(e)
    bad += not ok
    if not ok: print('DIFF', i, desc, '期望', e, '得', got)
print(f'{len(exp)} 个 case，不同 {bad}')
