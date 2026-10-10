#!/usr/bin/env python3
"""m3-batch10z-no-code-r1 攻方：两组 S3 日志按 (槽数, 前缀, R, 故障, persist, 动作串) 配对，报每组红几段、暴露几段、两组红或暴露不同的那几段。
用法：compare.py <名A> <名B> <A 的日志…> -- <B 的日志…>"""
import re, sys, collections
na, nb = sys.argv[1], sys.argv[2]
rest = sys.argv[3:]; cut = rest.index('--'); la, lb = rest[:cut], rest[cut + 1:]
def load(paths):
    out = {}
    for path in paths:
        for line in open(path, errors='replace'):
            if 'S3 label' not in line or 'record=skipped' in line: continue
            line = line[line.index('S3 label'):]
            f = dict(re.findall(r'(\w+)=([^ ]*)', line))
            out[(f['slots'], f['prefix'], f['r'], f['faults'], f['persist'], f.get('actions', '-'))] = f
    return out
A, B = load(la), load(lb)
def summary(name, rows):
    red = sum(f.get('hit') == 'true' for f in rows.values())
    exp = sum(f.get('first_exposed', '-') != '-' for f in rows.values())
    dbl = sum(f.get('first_dbl_c', '-') != '-' for f in rows.values())
    ref = sum(int(f.get('refused_steps', '0')) > 0 for f in rows.values())
    print(f"CMP {name}: histories={len(rows)} red={red} exposed={exp} dbl_c_or_other={dbl} refused_any={ref}")
summary(na, A); summary(nb, B)
common = sorted(set(A) & set(B)); diffs = 0
for k in common:
    a, b = A[k], B[k]
    if (a.get('hit'), a.get('first_exposed', '-') != '-') != (b.get('hit'), b.get('first_exposed', '-') != '-'):
        diffs += 1
        if diffs <= 12:
            print(f"CMPDIFF slots={k[0]} prefix={k[1]} r={k[2]} faults={k[3]} persist={k[4]} actions={k[5]} {na}: red={a.get('first_red')} exposed={a.get('first_exposed')} | {nb}: red={b.get('first_red')} exposed={b.get('first_exposed')}")
print(f"CMP paired={len(common)} differs={diffs}")
