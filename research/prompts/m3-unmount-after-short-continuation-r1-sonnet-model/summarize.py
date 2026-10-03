import re,sys
rows=[]
for f in sys.argv[1:]:
    for line in open(f):
        if not line.startswith('CELL'): continue
        head=re.match(r'CELL (\S+) (O\d+) (U\d+) (B\d+) K(\d+)',line).groups()
        if 'steps=' not in line:
            rows.append((*head,'-', '-','-','-','-','-', line.split('=>')[1].strip()[:40])); continue
        out=re.search(r'outcome=(Empty|Short\[(\w+)\|)',line)
        outcome=out.group(1) if out.group(1)=='Empty' else 'Short:'+out.group(2)
        q=re.search(r'queue=(\d)',line).group(1)
        fa=re.search(r'after-mount=d0\[free=(\d+)',line).group(1)
        un='ok' if 'unmount-ok' in line else 'ERR'
        rem=re.search(r'remount-(ok|err)',line).group(1)
        cyc=re.search(r'cycle-after-unmount-remount=(ok\(\w+ steps=\d+ raises=\d+\)|err\[[^\]]{0,40})',line)
        cfg=re.search(r'cfgF\[wrote=(\w+)\]-remount=(ok\(\w+ steps=\d+ raises=\d+\)|err\[[^\]]{0,40})',line)
        rows.append((*head,outcome,'q'+q,'free'+fa,'un-'+un,'rem-'+rem,(cyc.group(1) if cyc else '-')+' | '+(cfg.group(1)+' '+cfg.group(2) if cfg else '-'),''))
for r in rows: print(*r)
