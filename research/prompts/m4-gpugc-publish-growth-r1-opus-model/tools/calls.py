import sys
# 每次用户改动进各格的次数（次数 / publish_user_change_total 的次数）
cells = ['publish_the_change_once','settle_allocator_clone','records_of_each_leaf','allocator_make_room_for_record','allocator_release_find','allocator_record_for','first_entry_index_build','publish_admitted_total','judge_before_any_read_or_write']
for path in sys.argv[1:]:
    tag = path.rsplit('/',1)[-1]
    for line in open(path, errors='replace'):
        if 'name=probe_window' not in line: continue
        f = dict(kv.split('=',1) for kv in line.split() if '=' in kv)
        n = int(f['publish_user_change_total'].split(':')[0])
        if not n: continue
        parts = ' '.join(f"{c.replace('allocator_','a_').replace('_the_','_')}={int(f[c].split(':')[0])/n:.1f}" for c in cells)
        print(f"{tag} P={f['live_data_units']} {f['phase'][:11]} s={f['steps_done']} U={f['units_in_the_version']} R={f['allocation_records_in_the_version']} {parts}")
