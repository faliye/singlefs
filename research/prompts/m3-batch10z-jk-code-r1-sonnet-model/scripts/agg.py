import re, sys, glob, collections
rows = collections.defaultdict(list)
for path in sorted(glob.glob(sys.argv[1])):
    for line in open(path, encoding='utf-8'):
        if 'PROBEJK label' not in line: continue
        line = line[line.index('PROBEJK '):]
        f = dict(re.findall(r'(\w+)=(\S*)', line))
        if 'PROBEJK_DONE' in line or 'histories' in f: continue
        rows[(f['label'], f['arm'])].append(f)
for key in sorted(rows):
    g = rows[key]
    ok = [x for x in g if 'iso_mount' in x]
    def stat(n):
        v=[int(x[n]) for x in ok if x.get(n,'-') not in ('-','')]
        return f"{sum(v)/len(v):.1f}/{max(v)}" if v else "-"
    hit = sum(1 for x in ok if x['hit']=='true')
    pre = sum(1 for x in ok if 'preroot' in x.get('first_red',''))
    ref = sum(1 for x in ok if int(x['refused_steps'])>0)
    print(f"{key[0]:28s} arm={key[1]:12s} n={len(g)} mount_err={len(g)-len(ok)} hit={hit} preroot={pre} refused={ref} iso_mount={stat('iso_mount')} iso_first_raise={stat('iso_after_first_raise')} iso_max={stat('iso_max')} iso_end={stat('iso_end')} c_r={stat('c_r')} rev_in_c_r={stat('revived_in_c_r')} revived={stat('revived')}")
