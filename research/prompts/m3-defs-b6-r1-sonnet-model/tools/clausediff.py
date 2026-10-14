import sys, re, difflib
old, new = sys.argv[1], sys.argv[2]
def clauses(path):
    out=[]
    for i,l in enumerate(open(path,encoding='utf-8').read().split('\n'),1):
        parts=re.split(r'(?<=[；。])', l)
        for p in parts:
            if p.strip(): out.append((i,p))
    return out
o=clauses(old); n=clauses(new)
sm=difflib.SequenceMatcher(None,[x[1] for x in o],[x[1] for x in n],autojunk=False)
for tag,i1,i2,j1,j2 in sm.get_opcodes():
    if tag=='equal': continue
    print(f'--- {tag} old[{o[i1][0] if i1<len(o) else "-"}..] new[{n[j1][0] if j1<len(n) else "-"}..]')
    for k in range(i1,i2): print(f'  OLD {o[k][0]}: {o[k][1]}')
    for k in range(j1,j2): print(f'  NEW {n[k][0]}: {n[k][1]}')
