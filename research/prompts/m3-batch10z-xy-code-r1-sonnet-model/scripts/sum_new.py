#!/usr/bin/env python3
"""sum_new.py <log...> : 汇总 NS 报告：被宽护卡住按预演结局分类，预演与真推环重试的对照（CMP）、各 flag、第一次被拒就是 N 的数。"""
import re, sys, collections
c = collections.Counter(); bad = []
for f in sys.argv[1:]:
    for line in open(f, errors='replace'):
        if 'DIAG label' not in line or 'PUDIAG' in line or 'CRASHDIAG' in line: continue
        for m in re.finditer(r'NS\[([^|]*?)\]\|?(pushed\d+|pushRef:[^|]*)?\|?(stillIso=\d+/\d+)?\|?(retry\S*?)?\|?(CMP\[[^\]]*\])?(?=\s|$|\|)', line):
            pass
        m = re.search(r'step=\d+[A-Za-z] (NS\[[^|]*)', line)
        if not m: continue
        ns = m.group(1)
        if ';S(' not in ns:
            c['NotWide'] += 1; continue
        c['Stuck'] += 1
        reh = re.search(r'reh=([APR])', ns)
        k = reh.group(1) if reh else '?'
        c['reh=' + k] += 1
        for fl in ('!needs', '!iso', 'rNe', '!ways2', '!txt12', '!txt3', '!way1', '!way1b'):
            if fl in ns: c['flag' + fl] += 1; bad.append((f, line[:400]))
        cm = re.search(r'CMP\[([^\]]*)\]', line)
        if cm:
            c['withCMP'] += 1
            if cm.group(1):
                c['CMP-nonempty'] += 1; bad.append((f, line[:600]))
            else:
                r = re.search(r'\|(pushed\d+|pushRef:[^|]*)\|stillIso=(\d+)/(\d+)\|(retry\S*)', line)
                if r:
                    c[('pushed' if r.group(1).startswith('pushed') else r.group(1)[:20], 'retryOk' if r.group(4).startswith('retryOk') else ('retryQ' if r.group(4).startswith('retryQ') else 'retryNS'), k)] += 1
                    if int(r.group(2)) > 0: c['stillIso>0'] += 1
for k, v in sorted(c.items(), key=lambda x: str(x[0])): print(v, k)
print('--- 异常行（至多 12 条）')
for f, l in bad[:12]: print(f.split('/')[-1], l)
