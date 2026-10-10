import sys, re, collections
# 同一段历史（盘、前缀、中间、R、故障、用户动作串）上两臂逐段比：第一次被拒的步号、红、R 在环里时 C 里 R 那一槽露出的最多槽数。
rows = collections.defaultdict(dict)
for path in sys.argv[1:]:
    for line in open(path):
        if not line.startswith('S2 '):
            continue
        f = dict(m.groups() for m in re.finditer(r'(\w+)=(\S*)', line))
        k = (f['slots'], f['prefix'], f['mid'], f['r'], f['faults'], f['actions'])
        rows[k][f['arm']] = f
def step(f):
    fr = f.get('first_refused', '-')
    return None if fr in ('-', None) else int(re.match(r'\d+', fr).group())
agg = collections.OrderedDict()
for k, arms in rows.items():
    if 'tri' not in arms or 'nodbl' not in arms:
        continue
    t, n = arms['tri'], arms['nodbl']
    g = agg.setdefault(k[:5], collections.Counter())
    g['n'] += 1
    if 'mount' in t or 'mount' in n:
        g['mount_err'] += 1; continue
    st, sn = step(t), step(n)
    if st is not None and (sn is None or st < sn): g['tri_refused_earlier'] += 1
    if sn is not None and (st is None or sn < st): g['nodbl_refused_earlier'] += 1
    g['tri_red'] += t['hit'] == 'true'; g['nodbl_red'] += n['hit'] == 'true'
    g['tri_exposed'] += int(t['exposed_max']) > 0; g['nodbl_exposed'] += int(n['exposed_max']) > 0
    g['dbl_tri_max'] = max(g['dbl_tri_max'], int(t['dbl_mount']))
    g['iso_tri_max'] = max(g['iso_tri_max'], int(t['iso_mount'])); g['iso_nodbl_max'] = max(g['iso_nodbl_max'], int(n['iso_mount']))
for k, g in agg.items():
    sl, pre, mid, r, fa = k
    keys = ['n', 'mount_err', 'tri_refused_earlier', 'nodbl_refused_earlier', 'tri_red', 'nodbl_red', 'tri_exposed', 'nodbl_exposed', 'iso_tri_max', 'iso_nodbl_max', 'dbl_tri_max']
    print(f"PAIR slots={sl} prefix={pre} mid={mid or '-'} r={r} faults={fa} " + ' '.join(f"{x}={g[x]}" for x in keys))
