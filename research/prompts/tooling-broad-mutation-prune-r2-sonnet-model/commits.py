import json,subprocess,collections,hashlib,re,sys,statistics
sys.path.insert(0,'.')
from lex2 import *
R='/home/fy5090/code/singlefs'
rows=json.load(open('rows.json'))
listfiles=sorted(set(r['file'] for r in rows))
fileTotal=collections.Counter(r['file'] for r in rows)
# HEAD per (file,name): mutants, number of distinct fn instances
inst=collections.defaultdict(set);mut=collections.Counter()
for r in rows:
    if r['fn']:
        inst[(r['file'],r['fn'])].add(r['fnstart']);mut[(r['file'],r['fn'])]+=1
def git(*a):
    return subprocess.run(['git',*a],capture_output=True,cwd=R).stdout.decode('utf8',errors='replace')
shas=git('log','--no-merges','-150','--format=%h','--','crates').split()
def fnsig(s):
    fs=fn_spans(s);out=[]
    for f in fs:
        body=s[f['start']:f['end']]
        out.append((f['name'],hashlib.sha1(re.sub(r'\s+','',body).encode()).hexdigest()))
    return out
def outside(s):
    fs=fn_spans(s);t=tokens(s)
    spans=[(f['hdr'],f['end']) for f in fs]
    out=[]
    for kind,tx,a,b in t:
        if any(x<=a<y for x,y in spans): continue
        out.append(s[a:b])
    return ' '.join(out)
res=[]
for c in shas:
    ch=git('diff','--name-only',c+'^',c,'--','crates').split()
    lf=[f for f in ch if f in fileTotal]
    if not lf: continue
    eff=0;fl_own=0;fn_own=0;nfn=0;out_files=0;testfn=0;changedfiles=[];nonlist=[f for f in ch if f not in fileTotal]
    for f in lf:
        new=git('show',f'{c}:{f}');old=git('show',f'{c}^:{f}')
        if not old: 
            fl_own+=fileTotal[f];fn_own+=fileTotal[f];eff+=fileTotal[f];continue
        so=collections.Counter(fnsig(old));sn=fnsig(new)
        cnt=collections.Counter(sn)
        # changed new fns: multiset diff
        diff=cnt-so
        chg=collections.Counter()
        for (name,h),k in diff.items(): chg[name]+=k
        mm=0
        for name,k in chg.items():
            nfn+=k
            key=(f,name)
            if key in inst:
                mm+=mut[key]*min(k,len(inst[key]))/len(inst[key])
        fn_own+=mm
        fl_own+=fileTotal[f]
        oc=outside(old)!=outside(new)
        if oc: out_files+=1
        eff+= fileTotal[f] if oc else mm
        changedfiles.append((f.split('/')[-1],round(mm),fileTotal[f],oc))
    res.append(dict(eff=eff,c=c,files=len(lf),nonlist=len(nonlist),fl_own=fl_own,fn_own=round(fn_own),nfn=nfn,out_files=out_files,detail=changedfiles))
json.dump(res,open('commits.json','w'),ensure_ascii=False)
tot=len(rows)
print('commits touching list files',len(res),'of',len(shas))
for r in res: print(r['c'],'files',r['files'],'fns_changed(all,incl tests/nonlisted names)',r['nfn'],'file-own',r['fl_own'],'(%.1f%%)'%(100*r['fl_own']/tot),'fn-own',r['fn_own'],'(%.1f%%)'%(100*r['fn_own']/tot),'outside-fn-changed files',r['out_files'],'effective(F2)',round(r['eff']),'(%.1f%%)'%(100*r['eff']/tot))
import json as _j
_r=[x for x in res if x['c']!='53c9f384']
for _k in ('fl_own','fn_own','eff'):
    _v=sorted(x[_k] for x in _r)
    print('EXCL-53c9f384 (first import) n=%d %s median %d (%.1f%%) p25 %d p75 %d min %d max %d'%(len(_r),_k,statistics.median(_v),100*statistics.median(_v)/tot,_v[len(_v)//4],_v[3*len(_v)//4],_v[0],_v[-1]))
print('median effective %.1f%%'%(100*statistics.median(r['eff'] for r in res)/tot));print('median file-own %.1f%%  median fn-own %.1f%%'%(100*statistics.median(r['fl_own'] for r in res)/tot,100*statistics.median(r['fn_own'] for r in res)/tot))
