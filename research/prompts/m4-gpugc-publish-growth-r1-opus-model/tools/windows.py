import sys, re
# 读 probe_window 行，打每个窗口每次用户改动的几格毫秒与每 MiB 单价、次缺页
prev = {}
for path in sys.argv[1:]:
    tag = path.rsplit('/',1)[-1]
    last_faults = None
    for line in open(path, errors='replace'):
        if 'name=probe_window' not in line: continue
        f = dict(kv.split('=',1) for kv in line.split() if '=' in kv)
        def cell(n):
            c, s, a = f[n].split(':'); return int(c), float(s), int(a)
        n = cell('publish_user_change_total')[0]
        mib = int(f['unit_bytes_in_the_version'])/2**20
        faults = int(f['minor_faults_total'])
        key = (tag, f['live_data_units'])
        dfaults = faults - prev.get(key, faults)
        prev[key] = faults
        ms = lambda name: cell(name)[1]*1000/n if n else 0
        ci = ms('clone_into_versions_published'); cu = ms('carried_unit_clone')
        cu_mib = cell('carried_unit_clone')[2]/2**20/n if n else 0
        pc = cell('positive_control_extra_units_clone')
        pc_ms_per_mib = (pc[1]*1000/(pc[2]/2**20)) if pc[2] else 0
        print(f"{tag} P={f['live_data_units']} {f['phase'][:11]} s={f['steps_done']} n={n} U={f['units_in_the_version']} MiB={mib:.1f} R={f['allocation_records_in_the_version']} "
              f"clone_into={ci:.2f}ms ({ci/mib:.3f}/MiB) carried={cu:.2f}ms ({cu/cu_mib if cu_mib else 0:.3f}/MiB) replace={ms('replace_current_dropping_old'):.2f} drop={ms('apparatus_drop_of_the_published'):.2f} "
              f"pc={pc_ms_per_mib:.3f}/MiB faults/n={dfaults/n if n else 0:.0f} resolve={ms('settle_plan_resolve'):.2f} feib={ms('first_entry_index_build'):.2f}")
