import sys
# publish_user_change_total 减去它的直接子格（checks、once、clone_into、replace、relocate、discard），每次用户改动的毫秒
for path in sys.argv[1:]:
    tag = path.rsplit('/',1)[-1]
    for line in open(path, errors='replace'):
        if 'name=probe_window' not in line: continue
        f = dict(kv.split('=',1) for kv in line.split() if '=' in kv)
        g = lambda n: [float(x) for x in f[n].split(':')]
        n = g('publish_user_change_total')[0]
        if not n: continue
        tot = g('publish_user_change_total')[1]
        kids = ['checks_before_the_publish','publish_the_change_once','clone_into_versions_published','replace_current_dropping_old','relocate_kept_for_snapshots','discard_after_the_change']
        s = sum(g(k)[1] for k in kids)
        adm = g('publish_admitted_total'); once = g('publish_the_change_once')
        print(f"{tag} P={f['live_data_units']} {f['phase'][:11]} s={f['steps_done']} n={int(n)} total={tot*1000/n:.2f} outside_children={(tot-s)*1000/n:.2f} ({(tot-s)/tot*100:.1f}%) once_calls={int(once[0])} admitted_calls={int(adm[0])}")
