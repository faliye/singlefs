import re,sys
src=open(sys.argv[1],encoding='utf8').read().split('\n')
res=sys.argv[2]
# label map: lines of `Name => "label",` within label fn
lab={}
for i,l in enumerate(src):
    m=re.match(r'\s+(\w+) => "([^"]+)",\s*$',l)
    if m and 6000<i<6200: lab[m.group(1)]=m.group(2)
# find test
start=next(i for i,l in enumerate(src) if 'let formal_run_units' in l)
end=next(i for i,l in enumerate(src) if i>start and 'let mut controlled_in_some_unit' in l)
units=[];cur=None
for l in src[start:end]:
    m=re.search(r'scripts/replay.sh` 第 ([\d、]+) 行；产物 `([^`]+)`，(\d+) 条',l)
    if m: cur={'line':m.group(1),'prod':m.group(2),'n':int(m.group(3)),'crit':[]};units.append(cur);continue
    m=re.match(r'\s+Criterion::(\w+),?\s*$',l)
    if m and cur is not None: cur['crit'].append(m.group(1))
bad=0
for u in units:
    f=res+'/'+u['prod']
    got=set()
    for l in open(f,encoding='utf8'):
        m=re.search(r'name=control criterion=(\S+) ',l)
        if m: got.add(m.group(1))
    exp=set(lab[c] for c in u['crit'])
    ok = got==exp and len(u['crit'])==len(exp)==u['n']
    print(u['line'],u['prod'][47:],len(u['crit']),len(got),'OK' if ok else 'DIFF', sorted(got^exp) if not ok else '')
    bad+= (not ok)
print('units',len(units),'bad',bad)
