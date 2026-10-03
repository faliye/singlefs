import json,random,collections,statistics
rows=json.load(open('rows.json'))
fn=collections.Counter();fnfile={}
for r in rows:
    if r['fn']:
        k=(r['file'],r['fn'],r['fnstart']);fn[k]+=1;fnfile[k]=r['file']
filetot=collections.Counter(r['file'] for r in rows)
keys=list(fn)
print('functions with mutants',len(keys),'files',len(filetot))
# unit costs
caught_full=72.4; unv=0.9
mix_full=(241*72.4+39*0.9)/280
mix_T=(241*22.8+39*0.9)/280
print('steady-state mean s/mutant: full-run %.1f  T-first %.1f'%(mix_full,mix_T))
random.seed(1)
def stats(v): 
    v=sorted(v);return 'mean %.0f median %.0f p10 %.0f p90 %.0f'%(statistics.mean(v),statistics.median(v),v[int(len(v)*.1)],v[int(len(v)*.9)])
for k in (1,5,20):
    A=[];B=[];Bsame=[]
    for _ in range(20000):
        pick=random.sample(keys,k)
        A.append(sum(fn[x] for x in pick))
        files={fnfile[x] for x in pick}
        B.append(sum(filetot[f] for f in files))
    print('k=%d own-functions-only A: %s'%(k,stats(A)))
    print('     whole touched files B:  %s'%stats(B))
    for lab,v in (('A',A),('B',B)):
        m=statistics.mean(v);md=statistics.median(v)
        print('       %s mean: %.2f h full-run / %.2f h T-first;  median %.2f h / %.2f h'%(lab,m*mix_full/3600,m*mix_T/3600,md*mix_full/3600,md*mix_T/3600))
# same-file best case: k functions in one file (largest-prob file)
# harness-T spill: 22.8% of caught are harness-T (55/241)
print('spill estimate if every harness-T mutant outside the touched files must be rejudged: 0.228*(5091-B)')
# time units for the 14-commit table
cs=[c for c in json.load(open('commits.json')) if c['c']!='53c9f384']
for c in sorted(cs,key=lambda x:x['eff']):
    print(c['c'],'changed fns',c['nfn'],'own-fn',c['fn_own'],'eff(F2)',round(c['eff']),'file-level',c['fl_own'],'-> hours full/T-first (eff): %.1f / %.1f ; file-level %.1f / %.1f'%(c['eff']*mix_full/3600,c['eff']*mix_T/3600,c['fl_own']*mix_full/3600,c['fl_own']*mix_T/3600))
