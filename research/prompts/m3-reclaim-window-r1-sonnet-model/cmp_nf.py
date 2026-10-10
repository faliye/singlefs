import sys,re
mine,arm=sys.argv[1],sys.argv[2]
out='/home/fy5090/code/singlefs/research/results/e139-tightened-floor-2026-10-04.out'
def kv(l): return dict(re.findall(r'(\w+)=(\S+)',l))
exp={}
for l in open(out):
    if l.startswith('E7RESULT name=w_drain_failure arm='+arm+' ') or l.startswith('E7RESULT name=w_broken_slot arm='+arm+' '):
        d=kv(l); nm='drain' if 'w_drain_failure' in l.split()[1] else 'broken'
        key=(nm,d['s'],d['c0'],d.get('failures','-'))
        exp[key]=d
bad=n=0
for l in open(mine):
    if not l.startswith('name='): continue
    d=kv(l); nm='drain' if l.startswith('name=drain') else 'broken'
    key=(nm,d['s'],d['c0'],d.get('failures','-'))
    e=exp[key]; n+=1
    for f in ('false_enospc_arm','max_publications','stalls','fakes','unrecoverable','cases','injected','write_after_delete_failed'):
        if f in d and e.get(f)!=d[f]:
            bad+=1; print('DIFF',key,f,'mine',d[f],'out',e.get(f))
print('compared rows',n,'field diffs',bad)
