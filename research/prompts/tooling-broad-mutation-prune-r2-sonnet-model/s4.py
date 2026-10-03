import sys,subprocess,collections,re,json
sys.path.insert(0,'.')
from lex2 import *
R='/home/fy5090/code/singlefs'
allf=subprocess.run(['git','ls-tree','-r','--name-only','HEAD','crates'],capture_output=True,cwd=R).stdout.decode().split()
rs=[f for f in allf if f.endswith('.rs') and re.match(r'crates/(singlefs-core|singlefs-checker|singlefs-format|singlefs-harness)/(src|tests)/',f)]
inh=collections.defaultdict(set);trt=collections.defaultdict(set)
traits=collections.Counter()
for f in rs:
    s=src(f);t=tokens(s);n=len(t)
    fs=fn_spans(s)
    for k,(kd,tx,a,b) in enumerate(t):
        if kd=='id' and tx=='trait' and k+1<n and t[k+1][0]=='id' and (k==0 or t[k-1][1]!='dyn'): traits[(f.split('/')[1],t[k+1][1])]+=1
        if kd=='id' and tx=='impl':
            # parse header until '{'
            j=k+1;hdr=[];d=0
            while j<n and not(t[j][0]=='p' and t[j][1]=='{' and d==0):
                if t[j][1]=='<' : d+=0
                hdr.append(t[j]);j+=1
            ids=[x[1] for x in hdr if x[0]=='id']
            if 'for' in ids:
                i=ids.index('for');tr=ids[i-1] if i>0 else '?';ty=ids[i+1] if i+1<len(ids) else '?'
                kind='trait'
            else:
                ty=[x for x in ids if x not in('unsafe','dyn')][-1] if ids else '?';kind='inh';tr=None
            # impl body
            e=j;dd=0
            while e<n:
                if t[e][0]=='p' and t[e][1]=='{': dd+=1
                elif t[e][0]=='p' and t[e][1]=='}':
                    dd-=1
                    if dd==0: break
                e+=1
            lo=t[j][2];hi=t[e][3] if e<n else len(s)
            # direct fn items inside impl at depth 1: approximate: fns whose start in span and not nested in another fn within span
            inside=[x for x in fs if lo<=x['hdr']<hi]
            for x in inside:
                (trt if kind=='trait' else inh)[(ty,x['name'])].add((f,tr))
local=[(k,v) for k,v in traits.items()]
print('user traits declared in crates (core/checker/format/harness):',len(traits),sorted(traits)[:12])
both=[(k,inh[k],trt[k]) for k in inh if k in trt]
print('(type,method) pairs that are both inherent and trait methods:',len(both))
cross=[(k,sorted({f for f,_ in inh[k]}),sorted({f for f,_ in trt[k]})) for k,_,_ in both if {f for f,_ in inh[k]}!={f for f,_ in trt[k]}]
print('... in different files:',len(cross))
for c in cross[:10]: print(c)
