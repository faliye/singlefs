import re,sys,os,collections
src=sys.argv[1]
mods=sorted(f[:-3] for f in os.listdir(src) if f.endswith('.rs'))
mods=[m for m in mods if m not in('lib',)]
rootmods=set(re.findall(r'^\s*(?:pub )?mod (\w+);',open(src+'/lib.rs').read(),re.M))
def split(text):
    m=re.search(r'^#\[cfg\(test\)\]\s*\nmod \w+ \{',text,re.M)
    if m: return text[:m.start()],text[m.start():]
    return text,''
def strip(t):
    t=re.sub(r'//[^\n]*','',t)
    return t
res={}
for m in mods:
    t=open(f'{src}/{m}.rs').read()
    prod,test=split(t)
    prod=strip(prod); test=strip(test)
    def refs(x):
        r=set(re.findall(r'\bcrate::(\w+)',x))
        r|=set(re.findall(r'\bsuper::(\w+)',x))
        # use crate::{a,b}
        for blk in re.findall(r'crate::\{([^}]*)\}',x,re.S):
            r|=set(re.findall(r'(?:^|,)\s*(\w+)',blk))
        return r
    res[m]=(refs(prod),refs(test),len(t.splitlines()))
roots=collections.defaultdict(set)
if __name__=='__main__':
    start=sys.argv[2:]
    seen=set();order=[]
    stack=list(start)
    while stack:
        x=stack.pop()
        if x in seen: continue
        seen.add(x);order.append(x)
        if x in res:
            for r in res[x][0]:
                if r in rootmods or r in res: stack.append(r)
    for x in sorted(seen):
        p=sorted(r for r in res[x][0] if r in rootmods) if x in res else []
        nonmod=sorted(r for r in res[x][0] if r not in rootmods and r not in res) if x in res else []
        print(x,res[x][2] if x in res else '?', 'PROD->',p,'ROOTITEMS->',nonmod,'TESTONLY->',sorted(set(r for r in res[x][1] if r in rootmods)-set(p)) if x in res else '')
