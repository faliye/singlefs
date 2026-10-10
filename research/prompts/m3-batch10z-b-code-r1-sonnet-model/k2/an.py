import re,sys,collections
def load(path):
    res={}; iso={}
    for l in open(path):
        if l.startswith('ISOSET '):
            m=re.match(r'ISOSET (.*?) (None|Some\(.*\))\s*$',l.rstrip('\n'))
            iso[m.group(1)]=m.group(2)
        if l.startswith('RESULT '):
            label=l[7:].split(': arm_c=')[0]
            d={}
            d['selP']=re.search(r'selected_is_P=(\w+)',l).group(1)=='true'
            d['Cvalid']=re.search(r'C_for_valid_R=(\S+)',l).group(1)!='None'
            m=re.search(r'被拒=Some\(\((\d+), "(.*?)"\)\)',l)
            d['refuse']=int(m.group(1)) if m else None
            d['why']=''
            if m:
                w=re.search(r'cause: (\w+)',m.group(2)); d['why']=w.group(1) if w else m.group(2)[:60]
            d['viol_step']=re.search(r'逐步checker首个违例=(.*?) R单元被覆盖',l).group(1)
            d['viol_w']=re.search(r'违例\(写后\)=(\[.*?\]) \|',l).group(1)
            d['unmount']=re.search(r'卸载=(做成|拒)',l).group(1)
            d['viol_u']=re.search(r'违例\(卸载后\)=(\[.*?\])\s*$',l.strip()).group(1)
            res[label]=d
    return res,iso
T,TI=load('sweep/today-all.log'); C,CI=load('sweep/c-all.log')
print('cases today',len(T),'c',len(C),'same labels',set(T)==set(C))
def fam(label):
    k='OW' if label.startswith(('OW','M-OW')) else 'SQ'
    if label.startswith('M-'): t=label.split(' ')[1]
    else: t=label.split(' ')[1]
    return k,t
def summarize(R,name):
    g=collections.defaultdict(list)
    for lab,d in R.items():
        k,t=fam(lab)
        key=(k,t,'selP' if d['selP'] else 'notP','Cvalid' if d['Cvalid'] else 'noC')
        g[key].append(d)
    print('==',name)
    for key,ds in sorted(g.items()):
        ref=[d for d in ds if d['refuse'] is not None]
        whys=collections.Counter((d['refuse'],d['why']) for d in ref)
        vw=sum(1 for d in ds if d['viol_w']!='[]' or d['viol_step']!='None'); vu=sum(1 for d in ds if d['viol_u']!='[]')
        un=collections.Counter(d['unmount'] for d in ds)
        print(key,'n=%d'%len(ds),'拒%d'%len(ref),dict(whys),'违例(逐步/写后)%d 违例(卸载后)%d'%(vw,vu),'卸载',dict(un))
summarize(T,'今天读盘臂'); summarize(C,'C 臂')
# isoset compare
same=diff=0; diffs=[]
for lab in T:
    a,b=TI.get(lab),CI.get(lab)
    if a is None or b is None: continue
    if a=='None' or b=='None': diffs.append((lab,a[:40],b[:40])); continue
    if a==b: same+=1
    else: diff+=1; diffs.append((lab,a,b))
print('隔离集合 两臂都做成抬F的例子：相同',same,'不同',diff)
print('一臂做成一臂没做成的例子数',sum(1 for d in diffs if ('None' in (d[1],d[2]))))
bad=[d for d in diffs if 'None' not in (d[1],d[2])]
print('两臂都做成但集合不同',bad)
# one-arm-None cases: which arm None
cnt=collections.Counter()
for lab,a,b in diffs:
    if a=='None' and b!='None': cnt['today None / C Some']+=1
    elif b=='None' and a!='None': cnt['C None / today Some']+=1
    elif a=='None' and b=='None': cnt['both None']+=1
print(dict(cnt))
