import glob,re,collections,os,sys
root=os.path.dirname(os.path.abspath(__file__))+'/../logs/'
non384=('i256','i288','g256','g512','g768','h2','h3','h4','pu-320','base-256','ctl-early')
c=collections.Counter(); where=collections.defaultdict(collections.Counter)
for f in sorted(glob.glob(root+'sweeps/*.log')+glob.glob(root+'final/*.log')):
    nm=os.path.basename(f)[:-4]
    if nm.startswith('ctl') or nm.startswith('base-raise') : continue
    geo='非384' if nm.startswith(non384) else '384'
    for line in open(f,errors='replace'):
        if 'DIAG' not in line or 'PUDIAG' in line or 'CRASHDIAG' in line: continue
        m=re.search(r'step=\d+[A-Za-z] (NS\[[^|]*)',line)
        if not m: continue
        ns=m.group(1)
        kind='Stuck' if ';S(' in ns else 'NotWide'
        c[(geo,kind)]+=1
        if kind=='Stuck':
            for fl in ('!needs','!iso','rNe'):
                if fl in ns: c[(geo,'flag'+fl)]+=1
            r=re.search(r'\|(pushed\d+|pushRef:[^|]*)\|stillIso=(\d+)/(\d+)\|(retry\S*)',line)
            if r:
                ok='retryOk' if r.group(4).startswith('retryOk') else 'retryNS'
                c[(geo,'Stuck后自动推环',ok)]+=1
                if ok=='retryNS': where[geo][nm]+=1
                if int(r.group(2))>0: c[(geo,'推后仍隔离')]+=1
for k,v in sorted(c.items()): print(v,k)
for g,w in where.items(): print(g,'推后仍被拒的来源',dict(w))
