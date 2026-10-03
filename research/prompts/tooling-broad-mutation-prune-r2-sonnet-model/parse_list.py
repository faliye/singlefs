import json,re,collections,sys
sys.path.insert(0,'.')
from lex2 import *
L=json.load(open('/home/fy5090/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4/list.json'))
files=sorted(set(e['file'] for e in L))
cache={}
for f in files:
    s=src(f);cache[f]=(s,line_starts(s),fn_spans(s))
rows=[]
for e in L:
    s,ls,fs=cache[e['file']]
    o=off(ls,e['start'][0],e['start'][1],s)
    enc=[x for x in fs if x['start']<=o<x['end']]
    enc.sort(key=lambda x:x['end']-x['start'])
    inner=enc[0] if enc else None
    rows.append(dict(seq=e['sequence'],pkg=e['package'],file=e['file'],name=e['name'],fn=(inner['name'] if inner else None),is_const=(inner['is_const'] if inner else None),fnstart=(inner['start'] if inner else None),depth=len(enc)))
json.dump(rows,open('rows.json','w'))
print(len(rows),len(files))
print('no enclosing fn',sum(1 for r in rows if r['fn'] is None))
print('const fn enclosing',sum(1 for r in rows if r['is_const']))
# cross-check name
bad=0
for r in rows:
    nm=r['name']
    m=re.search(r' in (.+)$',nm)
    if m: last=m.group(1)
    else:
        m2=re.match(r'.*?: replace (.+?)(?: -> .*)? with ',nm)
        last=m2.group(1) if m2 else None
    if last: last=last.split('::')[-1]
    if r['fn'] is not None:
        if last!=r['fn']:
            bad+=1
            if bad<8: print('MISMATCH',nm,r['fn'])
print('mismatch (name in the mutant name vs the function my parser found)',bad)
