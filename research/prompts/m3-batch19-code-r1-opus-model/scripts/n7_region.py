#!/usr/bin/env python3
"""n7_region.py <n7.txt>：开关压回 p、建 n 个之后 I-9.6 判不判红，与「p + n 大于 max(组合里树内最大号, 最大墓碑号)」这一预测逐点比；
数 I-9.6 不红的点里压回的号不是发过的最大号的有几个。组合的「树内最大号」取 COMBO 行 keys（所选根那棵树），checker 取的是走到的各根里的最大（见 c3,r0）。"""
import re, sys
combos = {}; match = mismatch = holds = holds_not_at_max = 0; examples = []
for line in open(sys.argv[1], encoding='utf-8'):
    if line.startswith('COMBO '):
        m = re.match(r'COMBO (\S+) txg=\d+ wm=(\d+) keys=(\[.*?\]) tombstones=(\[.*?\])', line)
        if m:
            keys = [k[0] for k in eval(m.group(3))]; tomb = eval(m.group(4))
            combos[m.group(1)] = (max(keys) if keys else 0, max(tomb) if tomb else 0)
    elif line.startswith('ON '):
        m = re.match(r'ON (\S+) p=(\d+) n=(\d+) (.*)', line)
        combo, p, n, rest = m.group(1), int(m.group(2)), int(m.group(3)), m.group(4)
        if 'refused' in rest or 'panic' in rest:
            continue
        max_key, max_tomb = combos[combo]
        held = 'i96=holds' in rest
        if held == (p + n > max_key and p + n > max_tomb): match += 1
        else: mismatch += 1; print('MISMATCH', combo, f'p={p} n={n}')
        if held:
            holds += 1
            if p != max(max_key, max_tomb):
                holds_not_at_max += 1
                if len(examples) < 6: examples.append(f'{combo} p={p} n={n} 树内最大号={max_key} 最大墓碑号={max_tomb}')
print(f'REGION 预测对上 {match} 对不上 {mismatch}；I-9.6 不红的点 {holds}，其中压回的号不是发过的最大号 {holds_not_at_max}')
print('REGION 例', examples)
