import sys,subprocess,collections,re,json,statistics
sys.path.insert(0,'.')
from lex2 import *
R='/home/fy5090/code/singlefs'
allf=subprocess.run(['git','ls-tree','-r','--name-only','HEAD','crates'],capture_output=True,cwd=R).stdout.decode().split()
rs=[f for f in allf if f.endswith('.rs')]
scope=[f for f in rs if re.match(r'crates/(singlefs-core|singlefs-checker|singlefs-format)/src/',f)]
print('scope files',len(scope))
T={};S={}
for f in rs:
    s=src(f);S[f]=s;T[f]=tokens(s)
# definitions in scope: (file,name,kind,item_token_range)
defs=[]
for f in scope:
    t=T[f];n=len(t)
    for k,(kd,tx,a,b) in enumerate(t):
        if kd=='id' and tx in('const','static'):
            j=k+1
            if j<n and t[j][0]=='id' and t[j][1]=='mut': j+=1
            if j<n and t[j][0]=='id' and t[j][1] not in('fn','unsafe','async','extern','_') and j+1<n and t[j+1][1]==':':
                name=t[j][1]
                # initializer: from '=' to matching ';' at depth 0
                e=j+2;d=0
                while e<n and not(t[e][0]=='p' and t[e][1]=='=' and d==0): 
                    if t[e][0]=='p' and t[e][1] in '([{': d+=1
                    if t[e][0]=='p' and t[e][1] in ')]}': d-=1
                    e+=1
                st=e+1;d=0;e2=st
                while e2<n and not(t[e2][0]=='p' and t[e2][1]==';' and d==0):
                    if t[e2][0]=='p' and t[e2][1] in '([{': d+=1
                    if t[e2][0]=='p' and t[e2][1] in ')]}': d-=1
                    e2+=1
                defs.append(dict(f=f,name=name,kind=tx,toks=t[st:e2]))
    for fs in fn_spans(S[f]):
        if fs['is_const']:
            toks=[x for x in t if fs['start']<=x[2]<fs['end']]
            defs.append(dict(f=f,name=fs['name'],kind='constfn',toks=toks))
print('defs',collections.Counter(d['kind'] for d in defs),'files with defs',len(set(d['f'] for d in defs)))
byname=collections.defaultdict(list)
for d in defs: byname[d['name']].append(d)
screaming=lambda n: n.upper()==n and any(c.isalpha() for c in n)
# names ambiguity for const fn
cf=[n for n,v in byname.items() if any(x['kind']=='constfn' for x in v)]
print('const fn names',len(cf),'common-named (defined in >=2 files):',sum(1 for n in cf if len(set(x['f'] for x in byname[n]))>=2))
def refs_of_tokens(toks,self_defs,mode):
    r=set()
    for kd,tx,a,b in toks:
        if kd=='id' and tx in byname:
            if mode=='sc' and not screaming(tx): continue
            r.add(tx)
    return r
def closure(startfiles,mode,item_level=True):
    # initial refs: all identifiers in start files
    reached=set();work=[]
    for f in startfiles:
        for nm in refs_of_tokens(T[f],None,mode): 
            for d in byname[nm]:
                if mode=='sc' and d['kind']=='constfn': continue
                work.append(d)
    seen=set();files=set(startfiles)
    while work:
        d=work.pop()
        key=(d['f'],d['name'],d['kind'])
        if key in seen: continue
        seen.add(key);files.add(d['f'])
        if item_level: toks=d['toks']
        else: toks=T[d['f']]
        for nm in refs_of_tokens(toks,None,mode):
            for d2 in byname[nm]:
                if mode=='sc' and d2['kind']=='constfn': continue
                work.append(d2)
    return files
core=[f for f in scope if '/singlefs-core/src/' in f]
res={}
for mode in ('sc','all'):
    for lvl in (True,False):
        sizes=[]
        for f in scope:
            c=closure([f],mode,lvl)
            sizes.append((len(c),f.split('/')[-1],f))
        sz=[x[0] for x in sizes]
        print('mode',mode,'item-level' if lvl else 'file-level','closure size over 1 start file: median',statistics.median(sz),'min',min(sz),'max',max(sz),'of',len(scope))
        coreonly=[len([x for x in closure([f],mode,lvl) if '/singlefs-core/' in x]) for f in core]
        print('   core-only files in closure from each core file: median',statistics.median(coreonly),'max',max(coreonly),'(core has %d)'%len(core))
        res[(mode,lvl)]=dict(zip([x[2] for x in sizes],sz))
# start from all core files together (T executes a big part)
for mode in ('sc','all'):
    for lvl in (True,False):
        c=closure(core,mode,lvl)
        print('start=all core files',mode,'item' if lvl else 'file','-> files',len(c),'in core',len([x for x in c if '/singlefs-core/' in x]),'format',len([x for x in c if 'format' in x]),'checker',len([x for x in c if '/singlefs-checker/' in x]))
# which files hold sc consts
print('SCREAMING const/static defs by file:')
cc=collections.Counter(d['f'].split('/')[-1] for d in defs if d['kind']!='constfn' and screaming(d['name']))
print(cc.most_common(12))
print('const fn defs by file:',collections.Counter(d['f'].split('/')[-1] for d in defs if d['kind']=='constfn').most_common(10))
