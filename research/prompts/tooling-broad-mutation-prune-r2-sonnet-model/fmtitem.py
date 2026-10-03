import sys,subprocess,collections,re,json
sys.path.insert(0,'.')
from lex2 import *
R='/home/fy5090/code/singlefs'
rows=json.load(open('rows.json'))
files=sorted(set(r['file'] for r in rows));tot=collections.Counter(r['file'] for r in rows)
fmt='crates/singlefs-format/src/lib.rs'
def constitems(s):
    t=tokens(s);d={}
    n=len(t)
    for k,(kd,tx,a,b) in enumerate(t):
        if kd=='id' and tx in('const','static') and k+2<n and t[k+1][0]=='id' and t[k+2][1]==':' and t[k+1][1].upper()==t[k+1][1]:
            e=k;dd=0
            while e<n and not(t[e][0]=='p' and t[e][1]==';' and dd==0):
                if t[e][0]=='p' and t[e][1] in '([{': dd+=1
                if t[e][0]=='p' and t[e][1] in ')]}': dd-=1
                e+=1
            d[t[k+1][1]]=' '.join(s[x[2]:x[3]] for x in t[k:e])
    return d,t
head,_=constitems(src(fmt))
# item-level closure from each mutated file over format consts (initializers referencing other format consts)
deps={n:{x for x in head if x!=n and re.search(r'\b'+x+r'\b',v)} for n,v in head.items()}
def clos(names):
    seen=set();w=list(names)
    while w:
        x=w.pop()
        if x in seen: continue
        seen.add(x);w.extend(deps.get(x,()))
    return seen
fileclos={}
for f in files:
    tk=tokens(src(f))
    used={tx for kd,tx,a,b in tk if kd=='id' and tx in head}
    fileclos[f]=clos(used)
print('avg format consts in item-closure per mutated file: %.1f of %d'%(sum(len(v) for v in fileclos.values())/len(files),len(head)))
cs=json.load(open('commits.json'))
out=[]
for x in cs:
    c=x['c']
    if c=='53c9f384': continue
    r=subprocess.run(['git','diff','--quiet',c+'^',c,'--',fmt],cwd=R)
    if r.returncode==0: continue
    old,_=constitems(subprocess.run(['git','show',f'{c}^:{fmt}'],capture_output=True,cwd=R).stdout.decode())
    new,_=constitems(subprocess.run(['git','show',f'{c}:{fmt}'],capture_output=True,cwd=R).stdout.decode())
    chg={n for n in set(old)|set(new) if old.get(n)!=new.get(n)}
    # names that exist only in HEAD closure space: changed consts that still exist (or removed ones can't be mapped to HEAD; count with removed too if name in head)
    hit=[f for f in files if fileclos[f]&chg or any(n in chg for n in fileclos[f])]
    # transitive: a changed const also changes dependents
    chg2=set(chg)
    for n in head:
        if clos([n])&chg: chg2.add(n)
    hit2=[f for f in files if fileclos[f]&chg2]
    mfile=sum(tot[f] for f in files)  # all
    mh=sum(tot[f] for f in hit2)
    out.append((c,len(chg),len(chg2),len(hit2),mh))
    print(c,'format consts changed',len(chg),'(+dependents %d)'%len(chg2),'files whose item-closure hits:',len(hit2),'of',len(files),'mutants',mh,'(%.1f%%)'%(100*mh/5091))
