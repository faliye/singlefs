import re,sys
rows=[]
for f in sys.argv[1:]:
    for line in open(f):
        if not line.startswith('CELL') or 'steps=' not in line: continue
        m=re.match(r'CELL (\S+) (O\d+) (U\d+) (B\d+) K(\d+) rw=(\d+) before=d0\[free=(\d+) .*?at-crash=d0\[free=(\d+) .*?=> steps=(\d+) raises=(\d+) queue=(\d) outcome=(\S+?)(\[.*?\])? after-mount=d0\[free=(\d+) unblocked=(\d+) deferred=(\d+) alloc=(\d+)\].*?(unmount-ok|unmount-err)(.*?)(remount-\S+)',line)
        if not m:
            print('NOMATCH',line[:100]); continue
        g=m.groups()
        disk,o,u,b,k,rw,fb,fc,steps,raises,q,out,_,fa,_,defd,alloc,un,unrest,rem=g
        x=re.search(r'pub (\d+)/(\d+)',unrest)
        miss=''
        if x: miss=6*int(x.group(1))-int(fa)
        rows.append((disk,o,u,b,k,fb,fc,steps,q,out,fa,un,x.group(0) if x else '',miss,rem[:22]))
print('disk O U B K free_before free_at_crash steps queue_left outcome free_after_mount unmount refused_at est_missing_slots remount')
for r in rows: print(*r)
