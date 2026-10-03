import re,os,glob,statistics,collections
V=os.path.expanduser('~/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4')
base={}
for l in open(V+'/logs/baseline.one-test.log',errors='replace'):
    m=re.match(r'\s*PASS \[\s*([\d.]+)s\]\s+\(\s*\d+/\d+\)\s+(.*?)\s*$',l)
    if m: base[m.group(2)]=float(m.group(1))
calm=[];pairs=[]
for f in glob.glob(V+'/logs/[0-9]*.one-test.log'):
    cur={}
    for l in open(f,errors='replace'):
        m=re.match(r'\s*PASS \[\s*([\d.]+)s\]\s+\(\s*\d+/\d+\)\s+(.*?)\s*$',l)
        if m: cur[m.group(2)]=float(m.group(1))
    slow=[(cur[k],base[k]) for k in cur if k in base and base[k]>=5]
    if len(slow)<10: continue
    med=statistics.median(a/b for a,b in slow)
    calm.append((os.path.basename(f).split('.')[0],med,max(a/b for a,b in slow),len(slow)))
print('logs with >=10 slow baseline tests passing:',len(calm))
c2=[x for x in calm if x[1]<=2]
print('of which median ratio <=2 (calm-looking):',len(c2),' their max single-test ratios:',sorted(round(x[2],1) for x in c2))
c3=[x for x in calm if x[1]>2]
print('median ratio >2:',len(c3),sorted((int(x[0]),round(x[1],1)) for x in c3))
