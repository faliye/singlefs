#!/usr/bin/env python3
"""m3-batch10z-no-code-r1 攻方：把同一批里 today / pre14 两臂的 S3 行按 (槽数, 前缀, R, 故障, persist, 动作串) 配对，
报每臂红几段、两臂红法不同的那几段、各项探针的最大值。用法：pair.py <批名> <日志…>（件号单数 today、双数 pre14 由件表次序定，
这里按日志里 ARM_TREE 不可见，改按件表：件表里 today 行在前、pre14 行在后，件数一半一半）。"""
import re, sys, collections
batch = sys.argv[1]; logs = sys.argv[2:]
half = len(logs) // 2
rows = {}
def key_of(line):
    f = dict(re.findall(r'(\w+)=([^ ]*)', line))
    return (f['slots'], f['prefix'], f['r'], f['faults'], f['persist'], f.get('actions', '-')), f
stats = collections.Counter()
maxima = collections.defaultdict(lambda: [0]*6)
for i, path in enumerate(sorted(logs, key=lambda p: int(re.search(r'-(\d+)\.log$', p).group(1)))):
    tree = 'today' if i < half else 'pre14'
    for line in open(path, errors='replace'):
        if 'S3 label' not in line:
            continue
        line = line[line.index('S3 label'):]
        if 'record=skipped' in line:
            stats[(tree, 'skipped')] += 1
            continue
        k, f = key_of(line)
        rows.setdefault(k, {})[tree] = f
        stats[(tree, 'histories')] += 1
        if 'mount' in f and f['mount'].startswith('Err'): stats[(tree, 'mount_err')] += 1
        if f.get('hit') == 'true': stats[(tree, 'red')] += 1
        if f.get('first_exposed', '-') != '-': stats[(tree, 'exposed')] += 1
        if f.get('first_dbl_c', '-') != '-': stats[(tree, 'dbl_c_or_other')] += 1
        if 'probe_max' in f:
            for j, v in enumerate(f['probe_max'].split('/')): maxima[tree][j] = max(maxima[tree][j], int(v))
        c = f.get('cnt_all', '0/0/0/0').split('/')
        for j, name in enumerate(['c_exempt_partial', 'c_exempt_partial_unprotected_slots', 'dup_key_wider', 'ab_exempt_partial']):
            if int(c[j]) > 0: stats[(tree, name)] += 1
        stats[(tree, 'refused_any')] += int(f.get('refused_steps', '0')) > 0
diff = []
for k, both in rows.items():
    if 'today' in both and 'pre14' in both:
        t, p = both['today'], both['pre14']
        if (t.get('hit') == 'true') != (p.get('hit') == 'true'):
            diff.append((k, t.get('first_red'), p.get('first_red')))
        stats['paired'] += 1
for tree in ('today', 'pre14'):
    print(f"PAIR batch={batch} tree={tree} " + ' '.join(f"{n}={stats[(tree, n)]}" for n in ['histories', 'skipped', 'mount_err', 'red', 'exposed', 'dbl_c_or_other', 'refused_any', 'c_exempt_partial', 'c_exempt_partial_unprotected_slots', 'dup_key_wider', 'ab_exempt_partial']) + f" probe_max={'/'.join(map(str, maxima[tree]))}")
print(f"PAIR batch={batch} paired={stats['paired']} red_differs={len(diff)}")
for k, t, p in diff:
    print(f"DIFF batch={batch} slots={k[0]} prefix={k[1]} r={k[2]} faults={k[3]} persist={k[4]} actions={k[5]} today_red={t} pre14_red={p}")
