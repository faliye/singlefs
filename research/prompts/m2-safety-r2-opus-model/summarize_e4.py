import re,sys,collections
rows=collections.defaultdict(dict); sizes=set(); viol=0; dels=collections.Counter(); delok=collections.Counter()
for l in open(sys.argv[1]):
    if not l.startswith('E4 '): continue
    m=re.match(r'E4 slots=(\d+) arm=(\S+) filler=(\S+) N=(\d+) grow_stop=(\S+) delete=(\S+) success_after=(\S+) ',l)
    s,a,f,n,gs,d,sa=int(m.group(1)),m.group(2),m.group(3),m.group(4),m.group(5),m.group(6),m.group(7)
    viol+=int(re.search(r'viol=(\d+)',l).group(1))
    sizes.add(s)
    code=('D!' if d!='ok' else '')+(sa.replace('Some(','').replace(')','') if sa!='None' else 'X')
    rows[(a,f)][s]=f'N{n}:{code}'
    if f=='inode': dels[a]+=1; delok[a]+= d=='ok'
sizes=sorted(sizes)
print('violations', viol)
print('cell = N(近满时放行的最大单元数):删后重写成功前的用户步数(0=立刻；X=8步内没成)；D!=截断这一次发布本身被准入拒')
print('arm | filler | '+' | '.join(map(str,sizes)))
for (a,f),r in rows.items(): print(f'{a} | {f} | '+' | '.join(r[s] for s in sizes))
print('截断（删）被放行的格数（每臂 9 个盘宽）:', {a: f'{delok[a]}/{dels[a]}' for a in dels})
