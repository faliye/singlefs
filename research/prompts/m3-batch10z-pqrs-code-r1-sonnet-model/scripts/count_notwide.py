import glob,re,collections,os
root=os.path.dirname(os.path.abspath(__file__))+'/../logs/'
non384=('i256','i288','g256','g512','g768','h2','h3','h4','pu-320','base-256','ctl-early')
c=collections.Counter()
for f in sorted(glob.glob(root+'sweeps/*.log')+glob.glob(root+'final/*.log')):
    nm=os.path.basename(f)[:-4]
    if nm.startswith('ctl') or nm.startswith('base-raise'): continue
    geo='非384' if nm.startswith(non384) else '384'
    for line in open(f,errors='replace'):
        if 'DIAG' not in line or 'PUDIAG' in line or 'CRASHDIAG' in line: continue
        for m in re.finditer(r'NS\[(\w+)(?:\([0-9,]*\))?;(S\(.*?,r(?:Eq|Ne)\)|N);hold=(\d+);isoC=(\d+);',line):
            stop,wide,hold,iso=m.groups()
            if wide=='N':
                if int(hold)>0 and int(iso)>0: c[(geo,'报NotWide却 C 有记着落点且隔离着槽，停因='+stop)]+=1
                elif int(hold)>0: c[(geo,'报NotWide，C 有记着落点但 0 槽隔离，停因='+stop)]+=1
            else:
                if stop!='Ceil': c[(geo,'报Stuck但停因不是Ceil')]+=1
print(dict(c) if c else '0 个')
for k,v in sorted(c.items()): print(v,k)
