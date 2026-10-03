import re,sys,glob
def load(path):
    d={}
    for line in open(path):
        m=re.match(r'DIRECT s=(\d+) history=(\S+) anchor=(\S+) checker_window=(\S*) step=(\S+) newest=(\d+) released_withheld_dev0=(\d+)',line)
        if m: d[m.group(5)]=int(m.group(7))
    return d
out=sys.argv[1]
for s in (4,8):
    for h in ('k1','fillable'):
        off=load(f'{out}/d-{s}-{h}-off.out'); ring=load(f'{out}/d-{s}-{h}-ring-cw1.out')
        keys=list(off.keys())
        diffs=[(off[k]-ring[k],k) for k in keys]
        mx=max(diffs)
        print(f's={s} h={h} steps={len(keys)} max_per_step_diff={mx[0]} at step={mx[1]} mounted_diff={off["mounted"]-ring["mounted"]} peak_off={max(off.values())} peak_win={max(ring.values())} peakdiff={max(off.values())-max(ring.values())}')
