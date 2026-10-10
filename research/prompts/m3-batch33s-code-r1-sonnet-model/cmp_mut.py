import sys
old=[l.rstrip('\n') for l in open(sys.argv[1],encoding='utf-8')]
new=[l.rstrip('\n') for l in open(sys.argv[2],encoding='utf-8')]
print("old",len(old),"new",len(new))
# common prefix? 
import collections
oldn={}
for i,l in enumerate(old):
    p=l.split('\t'); oldn.setdefault(p[0],[]).append((i,p))
newn={}
for i,l in enumerate(new):
    p=l.split('\t'); newn.setdefault(p[0],[]).append((i,p))
added=[n for n in newn if n not in oldn]
removed=[n for n in oldn if n not in newn]
print("names added",len(added),"removed",len(removed))
changed=[]
for n in newn:
    if n in oldn:
        for (i,a),(j,b) in zip(oldn[n],newn[n]):
            if a!=b:
                cols=[k for k in range(max(len(a),len(b))) if (a[k] if k<len(a) else None)!=(b[k] if k<len(b) else None)]
                changed.append((n,i+1,j+1,cols,a[4:] ,b[4:], len(oldn[n]),len(newn[n])))
print("changed rows",len(changed))
colhist=collections.Counter(tuple(c[3]) for c in changed)
print(colhist)
for c in changed:
    n,i,j,cols,a,b,lo,ln=c
    print(i,j,cols,"cargo/test same" if a==b else "CARGO/TEST DIFF", n[:60])
print("ADDED:"); 
for n in added: print(newn[n][0][0]+1, n[:70])
print("REMOVED:",removed)
print("order same for unchanged tail?")
