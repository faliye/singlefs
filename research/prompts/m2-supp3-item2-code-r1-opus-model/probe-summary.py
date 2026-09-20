import sys,re,collections
mb=0; ic=0; walls=0; gap=[]; dis=collections.Counter(); ends=None
for l in open(sys.argv[1],errors='replace'):
    if l.startswith('历史 '): ends=l.strip()
    if not l.startswith('ATTACK_PROBE'): continue
    m=re.search(r'model_bound=Some\((\d+)\) impl_records=Some\((\d+)\)',l)
    if m:
        b,c=int(m[1]),int(m[2]); mb=max(mb,b); ic=max(ic,c)
        if b>812: gap.append(c)
    if 'AllocationRecordsExceedOneNode' in l: walls+=1
    d=re.search(r'model_disagreement=Some\((\w+)\)',l)
    if d: dis[d[1]]+=1
print(' max model bound',mb,'| max impl records',ic,'| wall refusals',walls,'| steps with bound>812',len(gap),'| impl count at those min',min(gap) if gap else None,'max',max(gap) if gap else None,'| model disagreements',dict(dis))
print(' ',ends)
