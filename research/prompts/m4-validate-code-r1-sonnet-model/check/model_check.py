import re,sys
P=32634
U64=(1<<64)-1
def needed(n): return max(1,-(-n//P))
def clamp(x,lo,hi): return max(lo,min(hi,x))
bad=0; checked={'TRUNC':0,'STEPS':0,'EXT':0,'FITS':0}
for line in open('probe_pure.out'):
    line=line.strip()
    if not line or line.startswith('test result'): continue
    kind=line.split()[0]
    kv=dict(re.findall(r'(\w+)=(\S+)',line))
    if kind=='TRUNC':
        e,s,n=int(kv['existing']),int(kv['s']),int(kv['n'])
        k=needed(n)-1
        if k>=e: exp='None'
        else:
            new=n-k*P; old=clamp(s-k*P,0,P)
            exp='None' if old==new else f'Some(({k}, {new}))'
        # rewrite field in rust print may contain spaces: re-extract
        m=re.search(r'rewrite=(None|Some\(\(\d+, \d+\)\))',line)
        got=m.group(1)
        carried = e<=needed(n)
        exp_txg,exp_n=(5,123) if carried else (9,n)
        ok = got==exp and kv['needed']==str(needed(n)) and kv['field_carried']==str(carried).lower() and kv['field_txg']==str(exp_txg) and kv['field_n']==str(exp_n)
        checked['TRUNC']+=1
        if not ok: bad+=1; print('BAD',line,'expected',exp,carried)
    elif kind=='STEPS':
        e,n,b=int(kv['existing']),int(kv['n']),int(kv['batch'])
        steps=[]
        cur=e
        while True:
            kept=min(cur,needed(n)); beyond=cur-kept
            if beyond<=b: steps.append(f'F{kept}'); break
            cur=cur-b; steps.append(f'B{cur}')
        exp=','.join(steps)
        checked['STEPS']+=1
        if kv['steps']!=exp: bad+=1; print('BAD',line,'expected',exp)
    elif kind=='EXT':
        e,s,n=int(kv['existing']),int(kv['s']),int(kv['n'])
        u=e-1; new=clamp(n-u*P,0,P); old=clamp(s-u*P,0,P)
        exp='None' if not old<new else f'Some(({u}, {new}))'
        m=re.search(r'rewrite=(None|Some\(\(\d+, \d+\)\))',line)
        checked['EXT']+=1
        if m.group(1)!=exp: bad+=1; print('BAD',line,'expected',exp)
    elif kind=='FITS':
        n=int(kv['n']); exp= n<=(U64//P)*P
        checked['FITS']+=1
        if kv['fits']!=str(exp).lower(): bad+=1; print('BAD',line,'expected',exp)
print('checked',checked,'bad',bad)
