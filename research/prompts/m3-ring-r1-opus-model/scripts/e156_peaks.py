#!/usr/bin/env python3
"""按 (s, rho, position, k) 取 Hh 观测点上 delta_real / delta_every 的峰值、h_que 的峰值，五个挂点并排。"""
import sys, collections
def load(path):
    cells = collections.OrderedDict()
    for line in open(path):
        if ' name=q1_observation ' not in line or ' family=Hh ' not in line:
            continue
        f = dict(tok.split('=', 1) for tok in line.split()[1:] if '=' in tok)
        key = (int(f['s']), f['rho'], f['position'], int(f['k']))
        c = cells.setdefault(key, dict(dr=0, de=0, hq=0, hd=0, n=0, i31=collections.Counter()))
        c['dr'] = max(c['dr'], int(f['delta_real'])); c['de'] = max(c['de'], int(f['delta_every']))
        c['hq'] = max(c['hq'], int(f['h_que'])); c['hd'] = max(c['hd'], int(f['h_dong'])); c['n'] += 1
        c['i31'][f['i31_on_hole']] += 1
    return cells
modes = sys.argv[1:]
data = [load(p) for p in modes]
print('s rho pos k | ' + ' | '.join(f'{m}: peakΔreal peakΔevery max_h_que n' for m in ['m0','m1','m2','m3','m4'][:len(modes)]))
for key in data[0]:
    row = []
    for d in data:
        c = d.get(key)
        row.append('missing' if c is None else f"{c['dr']} {c['de']} {c['hq']} {c['n']}")
    print(*key, '|', ' | '.join(row))
# 全体峰值
for m, d in zip(modes, data):
    for pos in ('front', 'back'):
        cs = [c for k, c in d.items() if k[2] == pos]
        print(f'TOTAL {m} position={pos} cells={len(cs)} peak_delta_real={max(c["dr"] for c in cs)} peak_delta_every={max(c["de"] for c in cs)} max_h_que={max(c["hq"] for c in cs)} observations={sum(c["n"] for c in cs)}')
