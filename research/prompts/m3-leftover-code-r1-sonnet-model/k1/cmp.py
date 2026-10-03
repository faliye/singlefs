import re,sys,difflib
old=open('/home/fy5090/code/singlefs/research/prompts/_m2-five-decisions-code-r1-diff.md').read().split('\n')
tree='/tmp/claude-1000/m3-leftover-code-r1/tree/'
# parse sections
secs={}
i=0
while i<len(old):
    m=re.match(r'### (\S+?):(\d+)-(\d+)（(\w+)',old[i])
    if m:
        # find fence
        j=i+1
        while j<len(old) and not old[j].startswith('```'): j+=1
        fence=old[j]; k=j+1
        body=[]
        while k<len(old) and old[k]!=fence.strip('rust').strip() and not (old[k].startswith('```') and len(old[k].strip('`'))==0 and len(old[k])>=len(fence.rstrip('rust'))): 
            body.append(old[k]); k+=1
        secs[(m.group(1),m.group(4))]=body
        i=k
    i+=1
def fn_text(path,name):
    src=open(tree+path).read().split('\n')
    # find "fn name" line
    for n,l in enumerate(src):
        if re.search(r'\bfn '+re.escape(name)+r'\b',l) or re.search(r'\b(enum|struct) '+re.escape(name)+r'\b',l):
            s=n
            # include preceding doc comments / attributes
            while s>0 and (src[s-1].lstrip().startswith('///') or src[s-1].lstrip().startswith('#[')): s-=1
            e=n
            ind=len(l)-len(l.lstrip())
            while e<len(src) and not (src[e]==' '*ind+'}' and e>n): e+=1
            return src[s:e+1]
    return None
for (path,name),body in secs.items():
    if not path.startswith('crates/singlefs-core') and not path.startswith('crates/singlefs-checker/'): continue
    if name in ('check_pool_image',): continue
    new=fn_text(path,name)
    if new is None: print('NOT FOUND',path,name); continue
    # strip doc-comment lines for comparision of code
    strip=lambda L:[x for x in L if not x.lstrip().startswith('//')]
    a=strip(body); b=strip(new)
    d=list(difflib.unified_diff(a,b,'0928','today',lineterm='',n=1))
    print('=====',path,name,'old',len(a),'new',len(b),'difflines',len(d))
    if len(d)<80:
        print('\n'.join(d))

print("\n######## whitespace/trailing-comma-insensitive comparison")
def norm(L):
    s=''.join(x for x in L if not x.lstrip().startswith('//'))
    s=re.sub(r'\s+','',s)
    s=re.sub(r',([)}\]])',r'\1',s)
    s=s.replace('return','').replace(';','')
    return s
for (path,name),body in secs.items():
    if not path.startswith('crates/singlefs-core') and not path.startswith('crates/singlefs-checker/'): continue
    new=fn_text(path,name)
    if new is None: continue
    a,b=norm(body),norm(new)
    print(name, 'SAME' if a==b else 'DIFF', len(a),len(b))
