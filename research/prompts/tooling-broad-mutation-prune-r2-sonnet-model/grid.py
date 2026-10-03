import re,os,glob
V=os.path.expanduser('~/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4')
base={}
for l in open(V+'/logs/baseline.one-test.log',errors='replace'):
    m=re.match(r'\s*PASS \[\s*([\d.]+)s\]\s+\(\s*\d+/\d+\)\s+(.*?)\s*$',l)
    if m: base[m.group(2)]=float(m.group(1))
pairs=[];logs={}
for f in glob.glob(V+'/logs/[0-9]*.one-test.log'):
    n=0
    for l in open(f,errors='replace'):
        m=re.match(r'\s*PASS \[\s*([\d.]+)s\]\s+\(\s*\d+/\d+\)\s+(.*?)\s*$',l)
        if m and m.group(2) in base:
            pairs.append((float(m.group(1)),base[m.group(2)],f));n+=1
print('PASS pairs (all baseline durations)',len(pairs),'logs',len(set(p[2] for p in pairs)))
for m in (3,5,10,20):
    for fl in (60,300):
        over=[p for p in pairs if p[0]>m*p[1]+fl]
        print('limit = %dx baseline + %ds: passing pairs over limit %d (%.2f%%) in %d logs'%(m,fl,len(over),100*len(over)/len(pairs),len(set(p[2] for p in over))))
import json,collections
res={}
for l in list(open(V+'/results.tsv'))[1:]:
    p=l.rstrip('\n').split('\t');res[int(p[0])]=p[3]
print('outcomes of the 22 non-caught(stage-one-only/timeout) mutants:',collections.Counter(o for o in res.values() if o in('timeout','stage-one-only')))
for m,fl in ((3,60),(5,60),(10,60),(10,300),(20,300)):
    over=set(os.path.basename(p[2]).split('.')[0] for p in pairs if p[0]>m*p[1]+fl)
    c=collections.Counter(res.get(int(s),'?') for s in over)
    print('limit %dx+%ds: logs with a passing test over limit by outcome:'%(m,fl),dict(c))
