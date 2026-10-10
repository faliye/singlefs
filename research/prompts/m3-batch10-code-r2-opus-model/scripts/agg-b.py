import re,glob,collections
D='/tmp/claude-1000/m3-batch10-code-r2-opus/b'
rec={}
for f in glob.glob(f'{D}/*.log'):
    for line in open(f):
        line=re.sub(r'^test b_\S* \.\.\. ','',line.rstrip('\n'))
        m=re.match(r'OPUS-B (\S+) point=(\S+) arm=\[(.*?)\] outcome=(\S+) hooks=(\d+) findings=(.*)$',line)
        if m:
            key=(m.group(2),m.group(3)); rec.setdefault(key,{})[m.group(1)]=(m.group(4),m.group(6))
print('records',len(rec), 'with both arms', sum(1 for v in rec.values() if len(v)==2))
bad=[(k,a,v) for k,vv in rec.items() for a,v in vv.items() if 'violations=[' in v[1] and ('violations=[]' not in v[1] or 'broken_at_or_above_F=[]' not in v[1])]
print('findings with violations or protected broken:',len(bad))
for b in bad[:20]: print('  ',b[0],b[1],b[2][0],b[2][1][:300])
# outcome comparison
cmp=collections.Counter()
for k,v in rec.items():
    if len(v)==2: cmp[(v['rr2_off'][0],v['rr2_on'][0])]+=1
for (off,on),n in sorted(cmp.items()): print(f'off={off} on={on} n={n}')
# refusals at steps
steps=collections.Counter()
for k,vv in rec.items():
    for a,v in vv.items():
        m=re.search(r'step=(\d+) refused=(\w+)',v[1])
        steps[(a,k[0],m.group(1) if m else 'none',m.group(2) if m else '')]+=1
for s,n in sorted(steps.items()): print('steps',s,n)
