import sys,re,collections,glob
c=collections.Counter()
for path in sorted(glob.glob(sys.argv[1])):
    for l in open(path):
        if 'OPUSF1' not in l: continue
        l=l[l.index('OPUSF1'):]
        l=re.sub(r'\((\d+), (\d+)\)',r'(\1,\2)',l)
        f=dict(re.findall(r'(\w+)=(\S+)',l))
        if 'mount' in f and f['mount'].startswith('Err'):
            c[(f['arm'],f['var'],'mount-refused:'+f['mount'][4:40])]+=1; continue
        form1 = f['selected']!=f['r'] and f['r_kind_at_mount']=='ValidRoot'
        steps=re.findall(r'(unmount|w\d+|rollback)=(\S+)',l)
        st=' '.join(a+':'+(b.split(':')[0].split('(')[0]+('/'+b.split(':')[1][:30] if b.startswith('Err') else '')) for a,b in steps)
        st=re.sub(r'Err@\d+','Err@n',st)
        c[(f['arm'],'form1' if form1 else 'not-form1('+f['selected']+')',f['var'],f['mode'],st,'over='+f['r_units_overwritten'],'checker='+f['checker'][:50])]+=1
for k,v in sorted(c.items()): print(v,*k)
