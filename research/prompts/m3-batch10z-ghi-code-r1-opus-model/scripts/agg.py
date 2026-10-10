import sys,re,glob,collections
rows=collections.OrderedDict()
for f in sorted(glob.glob(sys.argv[1])):
    for line in open(f):
        if 'OPUS10ZGHI arm=' not in line: continue
        line=line.split('OPUS10ZGHI ',1)[1]
        g=lambda k: re.search(k+r'=(\S+)',line).group(1)
        arm=g('arm')
        if '-persist-' in f and not arm.endswith('-persist'): arm+='-persist'
        key=(arm,g('k'),g('q'),g('r'),g('faults'))
        d=rows.setdefault(key,collections.Counter())
        d['histories']+=1
        if g('hit')=='true': d['hit']+=1
        if 'mount=Err' in line: d['mount_refused']+=1
        if re.search(r' [onsWFbu]=Err',line): d['step_refused']+=1
        if g('first_red_step')!='-': d['step_red']+=1
        m=re.search(r'first_preroot_red=(\S+)',line)
        if m and m.group(1)!='-':
            d['preroot_red']+=1
            d['red_root_txg='+re.search(r'txg(\d+)',m.group(1)).group(1)]+=1
for k,d in rows.items():
    print('arm=%s k=%s q=%s r=%s faults=%s '%k+' '.join('%s=%d'%(a,b) for a,b in sorted(d.items())))
