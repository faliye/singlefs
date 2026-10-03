import re,sys,collections
def load(path):
    rows={}
    for line in open(path):
        if line.startswith("name=c331grid "):
            key,_,rest=line[len("name=c331grid "):].partition(" verdict=")
            rows[("grid",key)]=rest.split(" ")[0]
        elif line.startswith("name=c331v4 "):
            head,_,tail=line.partition(" || ")
            head_key=" ".join(p for p in head.split(" ") if p.startswith("mount_k_slots") or p.startswith("mount_k_slot_timing") or p.startswith("mount_k_read_back"))
            key,_,rest=tail[len("name=c331grid "):].partition(" verdict=")
            hidden=re.search(r"hidden=(\S+)",key).group(1)
            rows[("v4",head_key+" hidden="+hidden)]=rest.split(" ")[0]
    return rows
if __name__=='__main__':
    for p in sys.argv[1:]:
        r=load(p)
        c=collections.Counter(r.values())
        lost=sum(v for k,v in c.items() if k.startswith('writable_LOST'))
        print(p.split('/')[-1],'rows',len(r),dict(c),'lost',lost)
