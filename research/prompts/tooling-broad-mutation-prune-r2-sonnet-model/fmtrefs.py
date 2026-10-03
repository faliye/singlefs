import sys,subprocess,collections,re,json
sys.path.insert(0,'.')
from lex2 import *
R='/home/fy5090/code/singlefs'
rows=json.load(open('rows.json'))
fmt='crates/singlefs-format/src/lib.rs'
t=tokens(src(fmt))
names=set()
for k,(kd,tx,a,b) in enumerate(t):
    if kd=='id' and tx in('const','static') and k+2<len(t) and t[k+1][0]=='id' and t[k+2][1]==':': names.add(t[k+1][1])
print('format consts',len(names))
files=sorted(set(r['file'] for r in rows))
tot=collections.Counter(r['file'] for r in rows)
ref=0;refm=0
direct={}
for f in files:
    tk=tokens(src(f))
    used={tx for kd,tx,a,b in tk if kd=='id' and tx in names}
    direct[f]=len(used)
    if used: ref+=1;refm+=tot[f]
print('mutated files',len(files),'directly referencing format consts',ref,'mutants in them',refm,'of',len(rows))
print('not referencing:',[ (f.split('/')[-1],tot[f]) for f in files if not direct[f]])
