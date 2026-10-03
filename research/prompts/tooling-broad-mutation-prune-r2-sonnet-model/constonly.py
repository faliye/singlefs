import sys,subprocess,collections,re,json
sys.path.insert(0,'.')
from lex2 import *
R='/home/fy5090/code/singlefs'
allf=subprocess.run(['git','ls-tree','-r','--name-only','HEAD','crates'],capture_output=True,cwd=R).stdout.decode().split()
rs=[f for f in allf if f.endswith('.rs')]
rows=json.load(open('rows.json'))
S={f:src(f) for f in rs}
FS={f:fn_spans(S[f]) for f in rs}
T={f:tokens(S[f]) for f in rs}
# const fns that have mutants
cfn=collections.defaultdict(list)  # name -> [(file,fnstart)]
mut=collections.Counter()
for r in rows:
    if r['is_const']: mut[(r['file'],r['fn'],r['fnstart'])]+=1
cands=set(mut)
print('const fns with mutants',len(cands),'mutants',sum(mut.values()))
def enclosing(f,off):
    enc=[x for x in FS[f] if x['start']<=off<x['end']]
    enc.sort(key=lambda x:x['end']-x['start'])
    return enc[0] if enc else None
# call sites by name across all rs
calls=collections.defaultdict(list)
for f in rs:
    t=T[f]
    for k,(kd,tx,a,b) in enumerate(t):
        if kd=='id' and k+1<len(t) and t[k+1][0]=='p' and t[k+1][1]=='(' :
            # skip definition: previous token is 'fn'
            if k>0 and t[k-1][1]=='fn': continue
            calls[tx].append((f,a,k>0 and t[k-1][1]=='.'))
const_only={};
status={}
def ctx(f,off):
    e=enclosing(f,off)
    if e is None: return 'const-item/other'
    if e['is_const']: return ('constfn',f,e['name'],e['start'])
    return 'runtime'
for (f,name,st) in cands:
    cs=calls.get(name,[])
    kinds=collections.Counter()
    for (cf,off,meth) in cs:
        c=ctx(cf,off)
        kinds['runtime' if c=='runtime' else ('item' if c=='const-item/other' else 'constfn')]+=1
    status[(f,name,st)]=kinds
# fixpoint: constfn callers that are themselves const-only count as const-context
def runtime_called(key,seen=()):
    f,name,st=key
    for (cf,off,meth) in calls.get(name,[]):
        c=ctx(cf,off)
        if c=='runtime': return True
        if c=='const-item/other': continue
        _,cf2,n2,s2=c
        k2=(cf2,n2,s2)
        if k2 in seen or k2==key: continue
        if runtime_called(k2,seen+(key,)): return True
    return False
none=0;constonly=0;rt=0;mconst=0;mnone=0;mrt=0
detail=[]
for key in cands:
    f,name,st=key
    ncall=len(calls.get(name,[]))
    if ncall==0: none+=1;mnone+=mut[key];detail.append(('nocall',name,mut[key]));continue
    if runtime_called(key): rt+=1;mrt+=mut[key]
    else: constonly+=1;mconst+=mut[key];detail.append(('const-only',name,f.split('/')[-1],mut[key]))
print('const fns: runtime-called',rt,'(mutants',mrt,') const-context-only',constonly,'(mutants',mconst,') no direct call syntactically',none,'(mutants',mnone,')')
for d in sorted(detail,key=lambda x:-x[-1])[:25]: print(d)
