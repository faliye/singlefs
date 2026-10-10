import sys, re, collections
# 按 (臂, 盘, 前缀, 中间, R, 故障) 归组：段数、红段、拒段（mount=Err 或 refused_steps>0）、iso_mount 最大、dbl_mount 最大
groups = collections.OrderedDict()
for path in sys.argv[1:]:
    for line in open(path):
        if not line.startswith('S2 '):
            continue
        f = dict(m.groups() for m in re.finditer(r'(\w+)=(\S*)', line))
        k = (f.get('arm'), f.get('slots'), f.get('prefix'), f.get('mid'), f.get('r'), f.get('faults'))
        g = groups.setdefault(k, dict(n=0, red=0, mount_err=0, refused=0, iso=[], dbl=[], first=set()))
        g['n'] += 1
        if 'mount' in f and f['mount'].startswith('Err'):
            g['mount_err'] += 1; g['first'].add(f['mount'][:50]); continue
        if f.get('hit') == 'true': g['red'] += 1
        if int(f.get('refused_steps', '0')) > 0:
            g['refused'] += 1; g['first'].add(f.get('first_refused', '')[:50])
        g['iso'].append(int(f['iso_mount'])); g['dbl'].append(int(f.get('dbl_mount', '0')))
for k, g in groups.items():
    arm, sl, pre, mid, r, fa = k
    print(f"arm={arm} slots={sl} prefix={pre} mid={mid} r={r} faults={fa} n={g['n']} red={g['red']} mount_err={g['mount_err']} refused={g['refused']} iso_mount_max={max(g['iso'], default='-')} dbl_mount_max={max(g['dbl'], default='-')} why={'|'.join(sorted(g['first'])) or '-'}")
