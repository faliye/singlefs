"""wall-summary.py <probe log>: for ATTACK_PROBE lines, list wall refusals of overwrite / first-file publishes with the impl's record count
and the model's bound of the (unchanged) current root; flags refusals where count + 16 <= 812 (the decided admission says the publish fits)."""
import sys,re,collections
fits=[];tot=0;dis=collections.Counter();rows=[]
for l in open(sys.argv[1],errors='replace'):
    if not l.startswith('ATTACK_PROBE'): continue
    d=re.search(r'model_disagreement=Some\((\w+)\)',l)
    if d: dis[d[1]]+=1
    if 'AllocationRecordsExceedOneNode' in l and re.search(r'kind=(PublishOverwrite|PublishFirstFile)',l):
        tot+=1
        m=re.search(r'seed=(\d+) step=(\d+).*model_bound=Some\((\d+)\) impl_records=Some\((\d+)\)',l)
        s,st,b,c=map(int,m.groups())
        if c+16<=812: fits.append((s,st,b,c))
print(f' publish wall refusals {tot}; of them count+16<=812 (decided admission says fits): {len(fits)}; model disagreements {dict(dis)}')
for f in fits[:5]: print('   seed %d step %d model_bound %d impl_records %d'%f)
