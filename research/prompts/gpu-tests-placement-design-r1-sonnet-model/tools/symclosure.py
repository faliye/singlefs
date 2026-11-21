import re,sys,os,collections,json
SRC=sys.argv[1]
SEEDS=json.loads(open(sys.argv[2]).read())  # {"module":["Name",...]} 
files=sorted(f[:-3] for f in os.listdir(SRC) if f.endswith('.rs'))
KW=r'(?:pub(?:\([a-z]+\))?\s+)?(?:unsafe\s+)?(?:async\s+)?(?:const\s+)?(fn|struct|enum|trait|const|static|type|impl|mod|macro_rules!)\b'
itemre=re.compile(r'^'+KW+r'\s*(<[^>]*>)?\s*([A-Za-z_][A-Za-z0-9_]*)?')
def strip(t):
    t=re.sub(r'"(?:\\.|[^"\\])*"','""',t)
    t=re.sub(r'//[^\n]*','',t)
    return t
items=[] # dict: mod,name,kind,start,end,tokens,test
for m in files:
    lines=open(f'{SRC}/{m}.rs').read().split('\n')
    starts=[]
    i=0
    n=len(lines)
    # find item starts at col 0
    cand=[]
    for i,l in enumerate(lines):
        if l and not l[0].isspace() and l[0] not in '}/)#]*{':
            mm=itemre.match(l)
            if mm and mm.group(1)!='mod':
                cand.append(i)
            elif mm and mm.group(1)=='mod':
                cand.append(i)
        elif l.startswith('#['):
            pass
    # attach attributes/docs: walk back
    def head(i):
        j=i
        while j>0 and (lines[j-1].startswith('///') or lines[j-1].startswith('#[') or lines[j-1].startswith('//') ):
            j-=1
        return j
    for k,i in enumerate(cand):
        h=head(i)
        e=(head(cand[k+1]) if k+1<len(cand) else n)
        # trailing '}' belongs; fine
        block='\n'.join(lines[h:e])
        mm=itemre.match(lines[i])
        kind=mm.group(1); name=mm.group(3) or ''
        if kind=='impl':
            hdr=lines[i]
            mt=re.match(r'^impl(?:\s*<[^>]*>)?\s+(?:(.+?)\s+for\s+)?([A-Za-z_][A-Za-z0-9_:]*)',hdr)
            if mt:
                name='impl '+mt.group(2).split('::')[-1]+(' (trait '+mt.group(1)+')' if mt.group(1) else '')
            else: name='impl ?'
        istest=any(x.startswith('#[cfg(test)]') for x in lines[h:i]) or (kind=='mod' and name=='tests')
        toks=set(re.findall(r'[A-Za-z_][A-Za-z0-9_]*',strip(block)))
        items.append(dict(mod=m,name=name,kind=kind,start=h+1,end=e,toks=toks,test=istest,loc=e-h,blk=strip(block)))

def split_impls(items):
    out=[]
    mre=re.compile(r'^    (?:pub(?:\([a-z]+\))?\s+)?(?:const\s+)?(?:unsafe\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)')
    for it in items:
        if it['kind']!='impl' or it['test'] or 'trait' in it['name']:
            out.append(it); continue
        lines=open(f"{SRC}/{it['mod']}.rs").read().split('\n')[it['start']-1:it['end']]
        ms=[k for k,l in enumerate(lines) if mre.match(l)]
        if not ms:
            out.append(it); continue
        def head(k):
            j=k
            while j>0 and (lines[j-1].strip().startswith('///') or lines[j-1].strip().startswith('#[') or lines[j-1].strip().startswith('//')): j-=1
            return j
        # shell: lines before first method head
        shell_end=head(ms[0])
        shell=dict(it); shell['loc']=shell_end+1; shell['toks']=set(re.findall(r'[A-Za-z_][A-Za-z0-9_]*',strip('\n'.join(lines[:shell_end+1])))); shell['blk']=strip('\n'.join(lines[:shell_end+1])); shell['shell']=True
        out.append(shell)
        for a,k in enumerate(ms):
            h=head(k); e=head(ms[a+1]) if a+1<len(ms) else len(lines)
            blk='\n'.join(lines[h:e])
            nm=mre.match(lines[k]).group(1)
            owner=it['name'].split(' ')[1]
            out.append(dict(mod=it['mod'],name=nm,kind='method',owner=owner,start=it['start']+h,end=it['start']+e-1,toks=set(re.findall(r'[A-Za-z_][A-Za-z0-9_]*',strip(blk))),test=False,loc=e-h,blk=strip(blk)))
    return out
if os.environ.get('METHODS'): items=split_impls(items)

import glob
if os.environ.get('TESTSDIR'):
    ttoks=set()
    for f in glob.glob(os.environ['TESTSDIR']+'/**/*.rs',recursive=True):
        ttoks|=set(re.findall(r'[A-Za-z_][A-Za-z0-9_]*',strip(open(f).read())))
    items.append(dict(mod='__tests__',name='__TESTS__',kind='fn',start=0,end=0,toks=ttoks,test=False,loc=0,blk=''))
# name map for non-impl, non-test items
defs=collections.defaultdict(list)
for idx,it in enumerate(items):
    if it['test'] or it['kind'] in('impl','mod'): continue
    defs[it['name']].append(idx)
byimpl=collections.defaultdict(list)
for idx,it in enumerate(items):
    if it['kind']=='impl' and not it['test']:
        tname=it['name'].split(' ')[1]
        byimpl[(it['mod'],tname)].append(idx)
        byimpl[(None,tname)].append(idx)

# file-level use map: name -> module
def usemap(m):
    text=strip(open(f'{SRC}/{m}.rs').read())
    # only top-of-file/column-0 'use' statements and also nested; take all 'use crate::...;'
    mp={}
    for stmt in re.findall(r'\buse\s+crate::([^;]+);',text,re.S):
        mm0=re.match(r'\s*(\w+)',stmt)
        if not mm0: continue
        mod=mm0.group(1)
        rest=stmt[len(mod):]
        for nm in re.findall(r'[A-Za-z_][A-Za-z0-9_]*',rest):
            mp.setdefault(nm,set()).add(mod)
    return mp
UM={m:usemap(m) for m in files}
UM['__tests__']={}
# per-item qualified refs
def qual(it,text_cache={}):
    return it['qual']
for it in items: it['qual']=set()
# recompute from source blocks: need block text again
src_cache={m:strip(open(f'{SRC}/{m}.rs').read()).split('\n') for m in files}
# note strip preserves line count? comments removed per-line; yes lines kept
for it in items:
    blk=it['blk']
    for mod,nm in re.findall(r'\bcrate::(\w+)::(\w+)',blk): it['qual'].add((mod,nm))
    for grp in re.findall(r'\bcrate::(\w+)::\{([^}]*)\}',blk):
        for nm in re.findall(r'[A-Za-z_][A-Za-z0-9_]*',grp[1]): it['qual'].add((grp[0],nm))
    for mod in files:
        for nm in re.findall(r'\b'+mod+r'::(\w+)',blk): it['qual'].add((mod,nm))
BLOCK=set(filter(None,os.environ.get('BLOCK','').split(',')))
keep=set();stack=[]
for idx,it in enumerate(items):
    if it['mod']=='__tests__': stack.append(idx)
for mod,names in SEEDS.items():
    for nm in names:
        c=[i for i in defs.get(nm,[]) if items[i]['mod']==mod]
        if not c: print('SEED NOT FOUND',mod,nm); continue
        stack+=c
reasons={}
typeitems={}
for idx,it in enumerate(items):
    if it['kind'] in('struct','enum','trait','type') and not it['test']: typeitems.setdefault((it['mod'],it['name']),[]).append(idx)
def owner_kept(it):
    if it['kind']!='method': return True
    return any(j in keep for j in typeitems.get((it['mod'],it['owner']),[]))
changed=True
visited=set()
while changed:
    changed=False
    while stack:
        i=stack.pop()
        if i in keep: continue
        keep.add(i); changed=True
        it=items[i]
        if it['kind'] in('struct','enum','trait','type'):
            for j in byimpl.get((it['mod'],it['name']),[]):
                if j not in keep: reasons.setdefault(j,('impl of',i)); stack.append(j)
        targets=set()
        for t in it['toks']:
            if t=='_' : continue
            for c in defs.get(t,[]):
                if items[c]['mod']==it['mod']: targets.add(c)
            for mod in UM[it['mod']].get(t,()):
                for c in defs.get(t,[]):
                    if items[c]['mod']==mod: targets.add(c)
        for mod,nm in it['qual']:
            for c in defs.get(nm,[]):
                if items[c]['mod']==mod: targets.add(c)
        for t in it['toks']:
            for c in defs.get(t,[]):
                if items[c]['kind']=='method' and owner_kept(items[c]): targets.add(c)
        for c in targets:
            if items[c]['name'] in BLOCK: continue
            if items[c]['kind']=='method' and not owner_kept(items[c]): continue
            if c not in keep and c!=i:
                reasons.setdefault(c,('from',i)); stack.append(c)
    # methods whose owner became kept after being skipped: re-scan
    for i in list(keep):
        pass
    # re-evaluate method candidates
    pend=[]
    for i in list(keep):
        it=items[i]
        targets=set()
        for t in it['toks']:
            for c in defs.get(t,[]):
                if items[c]['mod']==it['mod']: targets.add(c)
        for t in it['toks']:
            for c in defs.get(t,[]):
                if items[c]['kind']=='method' and owner_kept(items[c]): targets.add(c)
        for c in targets:
            if c in keep or items[c]['name'] in BLOCK: continue
            if items[c]['kind']=='method' and not owner_kept(items[c]): continue
            reasons.setdefault(c,('from',i)); stack.append(c); changed=True
bymod=collections.defaultdict(list)
for i in keep: bymod[items[i]['mod']].append(i)
tot=0
for m in sorted(bymod,key=lambda m:-sum(items[i]['loc'] for i in bymod[m])):
    s=sum(items[i]['loc'] for i in bymod[m]); tot+=s
    allprod=sum(it['loc'] for it in items if it['mod']==m and not it['test'])
    print(f'## {m}: kept {s} lines in {len(bymod[m])} items (file prod-item lines {allprod})')
print('TOTAL kept lines',tot)
if len(sys.argv)>3:
    for m in sys.argv[3].split(','):
        print('=====',m)
        for i in sorted(bymod[m],key=lambda i:items[i]['start']):
            it=items[i]; r=reasons.get(i)
            rr=(f"{items[r[1]]['mod']}::{items[r[1]]['name']}" if r else 'SEED')
            print(f"  {it['start']}-{it['end']} {it['kind']} {it['name']} ({it['loc']}) <- {rr}")
if False:
    for a in sorted(amb): print('AMB',a)
