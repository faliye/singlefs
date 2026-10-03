import sys,subprocess,collections,re,json,statistics
sys.path.insert(0,'.')
from lex2 import *
R='/home/fy5090/code/singlefs'
allf=subprocess.run(['git','ls-tree','-r','--name-only','HEAD','crates'],capture_output=True,cwd=R).stdout.decode().split()
scope=[f for f in allf if re.match(r'crates/(singlefs-core|singlefs-checker|singlefs-format)/src/.*\.rs$',f)]
tokmap={f:tokens(src(f)) for f in scope}
dtypes={}   # type name -> file
for f in scope:
    t=tokmap[f];n=len(t)
    for k,(kd,tx,a,b) in enumerate(t):
        if kd=='id' and tx in('struct','enum','union') and k+1<n and t[k+1][0]=='id':
            # look back for #[derive( within preceding attribute run
            j=k-1;found=False
            while j>=0:
                if t[j][0]=='id' and t[j][1] in('pub','crate','in','super','self'): j-=1;continue
                if t[j][1]==')' and t[j][0]=='p': j-=1;continue
                if t[j][1]=='(' and t[j][0]=='p': j-=1;continue
                if t[j][1]==']' and t[j][0]=='p':
                    # attribute: scan back to matching [
                    dd=0;q=j
                    while q>=0:
                        if t[q][1]==']' and t[q][0]=='p':dd+=1
                        elif t[q][1]=='[' and t[q][0]=='p':
                            dd-=1
                            if dd==0:break
                        q-=1
                    if any(x[1]=='derive' for x in t[q:j]): found=True
                    j=q-2; continue
                break
            if found: dtypes[t[k+1][1]]=f
print('derive types',len(dtypes),'in files',len(set(dtypes.values())),'of',len(scope))
rows=json.load(open('rows.json'));tot=collections.Counter(r['file'] for r in rows)
sizes=[];sz_m=[]
for f in sorted(tot):
    used={tx for kd,tx,a,b in tokmap[f] if kd=='id' and tx in dtypes}
    files={dtypes[x] for x in used}|{f}
    sizes.append(len(files));sz_m.append((len(files),tot[f]))
print('one-hop derive-definition files per mutated file: median %s min %s max %s (package has core 27 / checker 4 / format 1 files)'%(statistics.median(sizes),min(sizes),max(sizes)))
print('mutant-weighted mean %.1f files'%(sum(a*b for a,b in sz_m)/sum(b for a,b in sz_m)))
