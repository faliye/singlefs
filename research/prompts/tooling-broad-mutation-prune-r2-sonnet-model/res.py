import re,collections,statistics,os,json
V=os.path.expanduser('~/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4')
dur={}
for l in open(V+'/logs/baseline.one-test.log',errors='replace'):
    m=re.match(r'\s*PASS \[\s*([\d.]+)s\]\s+\(\s*\d+/\d+\)\s+(.*?)\s*$',l)
    if m: dur[m.group(2)]=float(m.group(1))
print('baseline per-test entries',len(dur))
v=sorted(dur.values())
print('baseline per-test: n=%d median %.3f p90 %.2f p99 %.2f max %.2f sum %.1f'%(len(v),statistics.median(v),v[int(len(v)*.9)],v[int(len(v)*.99)],v[-1],sum(v)))
print('tests >10s',sum(1 for x in v if x>10),'sum of them',sum(x for x in v if x>10))
res={}
for l in list(open(V+'/results.tsv'))[1:]:
    p=l.rstrip('\n').split('\t')
    res[int(p[0])]=p  # last wins
rows=json.load(open('rows.json'));byseq={r['seq']:r for r in rows}
cl=collections.Counter()
tcost=[];fullcost=[]
for s,p in res.items():
    out=p[3];T=p[5];b=float(p[6]);t=float(p[7])
    cl[out]+=1
    if out=='caught':
        key=T
        d=dur.get(T)
        cls='harness' if T.startswith('singlefs-harness') else ('core-lib' if T.startswith('singlefs-core ') else ('checker-lib' if T.startswith('singlefs-checker ') else 'other'))
        fullcost.append((b+t,cls,d,b,t))
print(cl, len(res))
by=collections.defaultdict(list)
for fc,cls,d,b,t in fullcost: by[cls].append((fc,d,b,t))
for cls,l in by.items():
    ds=[x[1] for x in l if x[1] is not None]
    print(cls,len(l),'full mean %.1f median %.1f'%(statistics.mean(x[0] for x in l),statistics.median(x[0] for x in l)),
      'build mean %.1f'%statistics.mean(x[2] for x in l),
      'T-baseline known %d mean %.1f median %.2f'%(len(ds),statistics.mean(ds) if ds else -1,statistics.median(ds) if ds else -1))
# T-only estimate = build + baseline duration of T (where known)
tonly=[b+d for fc,cls,d,b,t in fullcost if d is not None]
print('T-only (build + baseline T) n=%d mean %.1f median %.1f'%(len(tonly),statistics.mean(tonly),statistics.median(tonly)))
allfull=[fc for fc,_,_,_,_ in fullcost]
print('caught full mean %.1f'%statistics.mean(allfull))
# mutated-file by T class
mf=collections.defaultdict(collections.Counter)
for s,p in res.items():
    if p[3]=='caught':
        T=p[5];cls='harness' if T.startswith('singlefs-harness') else 'lib'
        mf[byseq[s]['file'].split('/')[-1]][cls]+=1
print(dict(mf))
