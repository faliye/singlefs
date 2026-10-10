import sys, re, collections
# 汇总 S2 行：按 (label, slots, r, faults, arm) 分组，数段数、红段数、拒段数、revived_in_c_r、r_old_in_ring、iso 读数
groups = collections.OrderedDict()
skipped = []
for path in sys.argv[1:]:
    for line in open(path, errors='replace'):
        m = re.search(r'S2 label=(\S+) arm=(\S+) .*', line)
        if not m: continue
        s = m.group(0)
        if 'record=skipped' in s:
            skipped.append(s[:200]); continue
        f = dict(kv.split('=', 1) for kv in re.findall(r'(\w+=[^ ]+)', s))
        key = (f['label'], f['arm'], f['slots'], f['r'], f.get('r_old_in_ring'), f['faults'], f.get('mid', ''))
        g = groups.setdefault(key, collections.Counter())
        g['histories'] += 1
        g['hit'] += f['hit'] == 'true'
        g['refused'] += f['refused_steps'] != '0'
        g['mount_err'] += f['selected'].startswith('mount') if 'selected' in f else 0
        g['revived_in_c_r'] += int(f.get('revived_in_c_r', 0))
        g['revived'] += int(f.get('revived', 0))
        g['iso_mount_max'] = max(g['iso_mount_max'], int(f.get('iso_mount', 0) if f.get('iso_mount', '0').isdigit() else 0))
        g['iso_max_max'] = max(g['iso_max_max'], int(f.get('iso_max', 0) if f.get('iso_max', '0').isdigit() else 0))
for key, g in groups.items():
    label, arm, slots, r, inring, faults, mid = key
    print(f"label={label} arm={arm} slots={slots} mid={mid or '-'} r={r} r_old_in_ring={inring} faults={faults} histories={g['histories']} hit={g['hit']} refused_histories={g['refused']} revived_in_c_r_sum={g['revived_in_c_r']}/{g['revived']} iso_mount_max={g['iso_mount_max']} iso_max_max={g['iso_max_max']}")
for s in skipped: print("SKIPPED", s)
