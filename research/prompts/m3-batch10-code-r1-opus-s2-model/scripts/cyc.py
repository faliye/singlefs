import sys,re
for f in sys.argv[1:]:
    lines=open(f,errors='replace').read().splitlines()
    st=[l for l in lines if l.startswith('KSTART')]
    print('==',f,st[0][:60] if st else '')
    for l in lines:
        if not l.startswith('KCYCLE') or 'plan=clean' in l: continue
        plan=re.search(r'plan=(\S+)',l).group(1); c=re.search(r'cycle=(\d+)',l).group(1)
        if 'REFUSED' in l: print(plan,c,'REFUSED',l.split('REFUSED')[1][:120].strip()); continue
        pat=re.search(r'pattern=(\S+)',l).group(1); fn=re.search(r'faulted_now=(\d+)',l).group(1); ct=re.search(r'cleared_total=(\d+)',l).group(1)
        ce=re.search(r'checker_end=(\[[^\]]*\])',l).group(1)
        stp=re.search(r'stop: (\w+ \{[^}]*\})',l)
        print(plan,c,pat,'faulted_now='+fn,'cleared='+ct,ce,stp.group(1) if stp else '')
