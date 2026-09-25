import re,collections,sys
rows=collections.defaultdict(lambda: collections.defaultdict(lambda:[0,0,0]))
sizes=set()
for l in open(sys.argv[1]):
    if not l.startswith('E3 '): continue
    m=re.match(r'E3 slots=(\d+) arm=(\S+) script=(\S+) admitted=(\d+) S4=(\d+) S4b=(\d+)',l)
    s,a=int(m.group(1)),m.group(2); sizes.add(s)
    r=rows[a][s]; r[0]+=int(m.group(4)); r[1]+=int(m.group(5)); r[2]+=int(m.group(6))
sizes=sorted(sizes)
print('格 = 放行的用户发布数 / S4 次数（放行之后下一次可写挂载被拒）/ S4b 次数（挂载放行、紧接着同样大小的覆盖写被拒），7 条脚本求和')
print('臂 | '+' | '.join(map(str,sizes))+' | S4 合计 | S4b 合计')
for a,r in rows.items():
    print(a+' | '+' | '.join(f'{r[s][0]}/{r[s][1]}/{r[s][2]}' for s in sizes)+f' | {sum(r[s][1] for s in sizes)} | {sum(r[s][2] for s in sizes)}')
