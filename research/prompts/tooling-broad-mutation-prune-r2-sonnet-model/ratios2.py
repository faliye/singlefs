import re,os,glob,statistics,collections,json
V=os.path.expanduser('~/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4')
base={}
for l in open(V+'/logs/baseline.one-test.log',errors='replace'):
    m=re.match(r'\s*PASS \[\s*([\d.]+)s\]\s+\(\s*\d+/\d+\)\s+(.*?)\s*$',l)
    if m: base[m.group(2)]=float(m.group(1))
print('baseline max',max(base.values()),'tests >=30s',sum(1 for v in base.values() if v>=30))
ratios=[];per=[];worst=[]
tot=0
for f in glob.glob(V+'/logs/[0-9]*.one-test.log'):
    cur={}
    for l in open(f,errors='replace'):
        m=re.match(r'\s*PASS \[\s*([\d.]+)s\]\s+\(\s*\d+/\d+\)\s+(.*?)\s*$',l)
        if m: cur[m.group(2)]=float(m.group(1))
    if not cur: continue
    tot+=1
    rs=[(cur[k]/base[k],k,cur[k],base[k]) for k in cur if k in base and base[k]>=5.0]
    for r in rs: ratios.append(r[0])
    if rs: 
        w=max(rs);worst.append((w[0],os.path.basename(f),w[1][:60],w[2],w[3]))
print('logs',tot,'(test,mutant) pairs with baseline>=5s:',len(ratios))
ratios.sort()
q=lambda p: ratios[int(len(ratios)*p)]
print('ratio of mutant-run PASS duration / baseline duration (baseline>=5s): median %.2f p90 %.2f p99 %.2f p99.9 %.2f max %.2f'%(statistics.median(ratios),q(.9),q(.99),q(.999),ratios[-1]))
for m in (2,3,4,5,10):
    print('share of passing pairs above %dx baseline: %.2f%%  (%d)'%(m,100*sum(1 for r in ratios if r>m)/len(ratios),sum(1 for r in ratios if r>m)))
worst.sort(reverse=True)
print('top worst logs',worst[:6])
# per log: share of logs having any test >Mx
for m in (2,3,5):
    print('logs with at least one passing test >%dx: %d of %d'%(m,sum(1 for w in worst if w[0]>m),len(worst)))
print('--- per-log median ratio (load signature)')
permed=[]
for f in sorted(glob.glob(V+'/logs/[0-9]*.one-test.log'),key=lambda x:int(os.path.basename(x).split('.')[0])):
    cur={}
    for l in open(f,errors='replace'):
        m=re.match(r'\s*PASS \[\s*([\d.]+)s\]\s+\(\s*\d+/\d+\)\s+(.*?)\s*$',l)
        if m: cur[m.group(2)]=float(m.group(1))
    rs=[cur[k]/base[k] for k in cur if k in base and base[k]>=5.0]
    if len(rs)>=10: permed.append((int(os.path.basename(f).split('.')[0]),statistics.median(rs),len(rs)))
print('logs with >=10 slow-test pairs',len(permed))
v=sorted(x[1] for x in permed)
print('per-log median ratio: min %.2f p25 %.2f median %.2f p75 %.2f max %.2f'%(v[0],v[len(v)//4],statistics.median(v),v[3*len(v)//4],v[-1]))
print([ (s,round(m,1)) for s,m,n in permed[::6]])
# a fixed test across logs
t='singlefs-harness::acquisition_writes_the_witnessed_journal_tail_so_two_crashed_acquisitions_still_witness_the_newest_publish'
for k in base:
    if k.startswith('singlefs-harness::acquisition_writes_the_witnessed_journal_t'): tt=k;break
vals=[]
for f in sorted(glob.glob(V+'/logs/[0-9]*.one-test.log'),key=lambda x:int(os.path.basename(x).split('.')[0])):
    for l in open(f,errors='replace'):
        m=re.match(r'\s*PASS \[\s*([\d.]+)s\]\s+\(\s*\d+/\d+\)\s+(.*?)\s*$',l)
        if m and m.group(2)==tt: vals.append((int(os.path.basename(f).split('.')[0]),float(m.group(1))))
print('fixed test',tt[:80],'baseline',base[tt],'n',len(vals),'min %.1f median %.1f max %.1f'%(min(v for _,v in vals),statistics.median(v for _,v in vals),max(v for _,v in vals)))
