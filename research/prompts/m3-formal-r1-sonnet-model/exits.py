import sys, re
src=open('/home/fy5090/code/singlefs/research/prompts/m3-formal-r1-sonnet-model/replay_model.py',encoding='utf-8').read()
lines=src.split('\n')
out=[]; in_replay=False; labels={}
for i,l in enumerate(lines,1):
    if l.startswith('def replay('): in_replay=True
    elif l.startswith('def '): in_replay=False
    if in_replay and l.strip().startswith('break'):
        ind=l[:len(l)-len(l.lstrip())]
        out.append(f"{ind}EXITS[{i}] += 1")
        labels[i]=(lines[i-2].strip()[:60], re.findall(r'# *([0-9-]+)',l))
    out.append(l)
src2='\n'.join(out).replace('class Err(Exception):','EXITS = __import__("collections").Counter()\nclass Err(Exception):',1).split('if __name__ == "__main__":')[0]
ns={}; exec(compile(src2,'m','exec'),ns)
EXITS=ns['EXITS']
dom=[(1,t,x,cm,o,l,ch,v) for t in (2,3,4) for x in (1,2) for cm in (True,False) for o in (1,2) for l in (True,False) for ch in ('H','B') for v in (True,)]
dom_v=[d[:7]+(False,) for d in dom if d[1]==3][:6]
res={}
for n in (1,2,3):
    EXITS.clear()
    ns['check_unconstrained'](n,dom+dom_v)
    res[n]=dict(EXITS)
allx=sorted(labels)
print('replay() 里 break 出口共',len(allx),'个')
for ln in allx:
    firstn=[n for n in (1,2,3) if ln in res[n]]
    print(f"model行{ln} Rust行注 {labels[ln][1]} 最小N={firstn[0] if firstn else '>3(未命中)'} 条件={labels[ln][0]}")
