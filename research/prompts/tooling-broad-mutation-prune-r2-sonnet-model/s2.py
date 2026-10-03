import sys,collections,json,re
sys.path.insert(0,'.')
exec(open('constgraph.py').read().split("core=[f for f in scope")[0])
rows=json.load(open('rows.json'));tot=collections.Counter(r['file'] for r in rows)
files=sorted(tot)
def one_hop(f):
    s={f}
    for nm in refs_of_tokens(T[f],None,'sc'):
        for d in byname[nm]:
            if d['kind']!='constfn': s.add(d['f'])
    return s
twohop=[];m2=0
for f in files:
    a=one_hop(f);b=closure([f],'sc',True)
    if b-a: twohop.append((f.split('/')[-1],len(a),len(b),tot[f])); m2+=tot[f]
print('mutated files where full item-level const closure has files beyond the one-hop set:',len(twohop),'of',len(files),'mutants',m2,'of 5091')
print(twohop[:12])
