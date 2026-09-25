import re,sys,collections
def ex(t):
    out=[]
    for tok in t.split(','):
        m=re.match(r'^(.+)x(\d+)$',tok)
        if m: out+= [m.group(1)]*int(m.group(2))
        else: out.append(tok)
    return out

lines=[l for l in open(sys.argv[1]) if l.startswith('E2 ')]
tab=collections.defaultdict(dict)
viol=0
sizes=set()
for l in lines:
    m=re.match(r'E2 slots=(\d+) arm=(\S+) script=(\S+) ',l)
    slots,arm,script=int(m.group(1)),m.group(2),m.group(3)
    sizes.add(slots)
    t1=re.search(r'\[remount,ow,remount,ow\]=(.*?) viol=(\d+)',l)
    t2=re.search(r'\[raiseF,ow\]=(.*?) viol=(\d+)',l)
    t3=re.search(r'\[rollbackAny,ow,remount,ow\]=(.*?) viol=(\d+)',l)
    va=int(re.search(r'viol_after_script=(\d+)',l).group(1))
    viol+=va+int(t1.group(2))+int(t2.group(2))+int(t3.group(2))
    r1=ex(t1.group(1))
    code=''
    if r1[0]=='ok' and r1[1]=='ok': code='W'   # remount then write ok
    elif r1[0]=='ok': code='M'                   # mount ok, write refused
    else: code='-'
    r2=ex(t2.group(1))
    code+= 'F' if (r2[0].startswith('ok') and r2[-1]=='ok') else ('f' if r2[0].startswith('ok') else '-')
    r3=[t3.group(1)]
    code+= 'B' if r3[0].startswith('rb-ok') else '-'
    tab[(arm,script)][slots]=code
sizes=sorted(sizes)
print('violations total:',viol)
print('arm | script | '+' | '.join(str(s) for s in sizes))
for (arm,script),row in tab.items():
    print(f'{arm} | {script} | '+' | '.join(row.get(s,'?') for s in sizes))
