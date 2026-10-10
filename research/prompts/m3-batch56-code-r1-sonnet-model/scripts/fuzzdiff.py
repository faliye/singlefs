import re,sys,collections
def load(p):
    d={}
    for l in open(p,encoding='utf-8'):
        l=l.rstrip('\n')
        k,rest=l.split(' | RECOVER ',1)
        rec,mnt=rest.split(' | MOUNT ',1)
        d[k]=(rec,mnt)
    return d
def member(s):
    m=re.search(r'detail: "([^"]*)"',s)
    if m: return m.group(1)
    m=re.search(r'(EntryNarrowerThanItsFieldTable|InodeNumberWatermarkRowMissingFromTheAccountingTree|[A-Z][A-Za-z]+)\b',s)
    return (s[:60] if not m else m.group(1))
new=load(sys.argv[1]); old=load(sys.argv[2])
assert new.keys()==old.keys()
print('images',len(new))
cnt=collections.Counter(); ex={}
tot=collections.Counter()
for k in new:
    nr,nm=new[k]; orr,om=old[k]
    tot[('rec',member(nr))]+=1
    ch=[]
    if nr!=orr: ch.append(('rec',member(orr),member(nr)))
    if nm!=om: ch.append(('mnt',member(om),member(nm)))
    for c in ch:
        cnt[c]+=1; ex.setdefault(c,k)
print('--- member distribution (new, recover):')
for k,v in tot.most_common(): print(v,k)
print('--- changed members (old -> new):')
for k,v in cnt.most_common(): print(v,k,'e.g.',ex[k])
print('total images with any change:',len({k for k in new if new[k]!=old[k]}))
