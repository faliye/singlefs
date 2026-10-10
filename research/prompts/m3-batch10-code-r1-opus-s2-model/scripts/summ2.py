import re,sys
for f in sys.argv[1:]:
    for line in open(f, errors='replace'):
        if line.startswith('KPLAN ') or line.startswith('KREFUSED '):
            lab = re.search(r'^K\w+ (.*?) n=', line).group(1)
            plan = re.search(r' plan=(\S+)', line).group(1)
            n = re.search(r' n=(\S+)', line).group(1)
            j = re.search(r' j=(\S+)', line).group(1)
            if line.startswith('KREFUSED'):
                why = re.search(r'writes_before_err=(\d+) (\w+)', line)
                print(f"{f.split('/')[-1][:-4]}|{lab}|n={n} j={j}|{plan}|REFUSED w={why.group(1)} {why.group(2)}")
                continue
            def g(k):
                m = re.search(r' '+k+r'=(\S+)', line); return m.group(1) if m else '-'
            ring = re.search(r'rereads=ring:(\w+)', line)
            ow = re.search(r'overwrite(\d+)_refused=(\w+)', line)
            print(f"{f.split('/')[-1][:-4]}|{lab}|n={n} j={j}|{plan}|ok ring={ring.group(1) if ring else '-'} F_row={g('floor_in_row')} raises={g('floor_raises')} ow={g('overwrites_done')}{'(refused@'+ow.group(1)+' '+ow.group(2)+')' if ow else ''} unmount={g('unmount')} faulted={g('faulted_slots')} allover={g('all_faulted_overwritten_at')} surv_bad={g('survivor_bad')} red={g('checker_red')}")
