import re,sys,collections
c=collections.Counter()
for l in open(sys.argv[1]):
    if not l.startswith('RESULT'): continue
    tag=l.split(' 挂载时')[0].replace('RESULT ','')
    sel=re.search(r'selected_is_P=(\w+)',l).group(1)
    cf=re.search(r'C_for_valid_R=(\S+)',l).group(1)
    ref='拒' if '被拒=Some' in l else '不拒'
    fv=re.search(r'逐步checker首个违例=(.*?) R单元被覆盖\(写后\)',l)
    fv=fv.group(1) if fv else 'n/a(被拒前)'
    vw=re.search(r'违例\(写后\)=(\[.*?\]) \|',l).group(1)
    un=re.search(r'卸载=(做成|拒)',l).group(1)
    vu=re.search(r'违例\(卸载后\)=(\[.*?\])\s*$',l.strip())
    vu=vu.group(1) if vu else '?'
    why=''
    m=re.search(r'cause: (\w+)',l)
    if ref=='拒' and m: why=m.group(1)
    key=(tag,'selP='+sel,'Cvalid='+('有' if cf!='None' else '无'),ref,why,'firstviol='+fv[:40],'写后viol='+vw[:30],'卸载='+un,'卸后viol='+vu[:30])
    c[key]+=1
for k,v in sorted(c.items()): print(v,k)
