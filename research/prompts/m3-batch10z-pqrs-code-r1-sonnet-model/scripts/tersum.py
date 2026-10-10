import sys,re,collections
c=collections.Counter(); ex={}
for f in sys.argv[1:]:
    for line in open(f,errors='replace'):
        if 'DIAG' not in line: continue
        d=line[line.index('DIAG'):]
        m=re.search(r'prefix=(\d+) r=\S+ faults=(\S+) actions=\w+ step=(\d+)\w (NS\[[^|]*\])\|([^|]*)\|stillIso=(\d+)/(\d+)\|(.*)',d)
        if not m: continue
        pre,fl,idx,ns,push,left,tot,retry=m.groups()
        kind='Stuck' if ';S(' in ns else 'NotWide'
        key=(f.split('/')[-1][:2],kind,re.sub(r'\d+','N',push)[:18],'left>0' if int(left)>0 else 'left0','retryOk' if retry.startswith('retryOk') else 'retryNS' if retry.startswith('retryNS') else retry[:20])
        c[key]+=1
        if key[1]=='Stuck' and key[4]!='retryOk': ex.setdefault(key,d[:420])
for k,v in sorted(c.items()): print(v,k)
for k,v in ex.items(): print('EX',k,v)
