import re,sys,collections
rows=[]
for f in sys.argv[1:]:
    for line in open(f, errors='replace'):
        if not (line.startswith('KPLAN ') or line.startswith('KREFUSED ')): continue
        lab=re.search(r'^K\w+ (.*?) n=', line).group(1); plan=re.search(r' plan=(\S+)', line).group(1)
        if line.startswith('KREFUSED'):
            why=re.search(r'writes_before_err=(\d+) (\w+)', line); rows.append((lab,plan,'REFUSED:'+why.group(2),'','','')); continue
        g=lambda k:(re.search(r' '+k+r'=(\S+)', line) or [None,'-'])[1]
        rows.append((lab,plan,'ok',g('checker_red'),g('survivor_bad'),g('i79_max_excess')))
print('histories', len(rows))
c=collections.Counter(r[2] for r in rows); print('outcomes', dict(c))
names=collections.Counter()
for r in rows:
    if r[3] not in ('','none','-'):
        for item in r[3].split(','):
            names[item.split('x')[0]]+=1
print('histories with checker red, by name', dict(names))
print('survivor bad histories', sum(1 for r in rows if r[4] not in ('','0','-')))
print('i79 excess>0', [(r[0][:30],r[1],r[5]) for r in rows if r[5] not in ('','0','-')])
for r in rows:
    if r[3] not in ('','none','-') and any(not x.startswith('I-3.1:more') for x in r[3].split(',')):
        print('NONMORE', r[0][:40], r[1], r[3])
