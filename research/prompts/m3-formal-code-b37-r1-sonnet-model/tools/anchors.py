import sys
tree=sys.argv[1]
f=tree+"/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs"
src=open(f,encoding="utf-8").read()
lines=src.split("\n")
def line_of(off): return src.count("\n",0,off)+1
lo=hi=None
for i,l in enumerate(lines,1):
    if "fn every_criterion_has_a_positive_control_and_every_control_is_red" in l: lo=i
    if lo and i>lo and l=="    }" : hi=i; break
print("test fn lines",lo,hi)
for n,row in enumerate(open(tree+"/crates/mutations.tsv",encoding="utf-8"),1):
    if row.startswith("#"): continue
    c=row.rstrip("\n").split("\t")
    if len(c)<6 or c[1]!="crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs": continue
    a=c[2].replace("\\n","\n")
    k=src.count(a)
    if k==1:
        s=src.index(a); l0=line_of(s); l1=line_of(s+len(a))
        if lo<=l0<=hi or lo<=l1<=hi:
            print("row",n,"anchor lines",l0,l1,"tests:",c[5],"|",c[0][:80])
    if n in (3383,3407): print("row",n,"anchor count",k,"lines",line_of(src.index(a)) if k else None,"test",c[5], "args",c[4])
