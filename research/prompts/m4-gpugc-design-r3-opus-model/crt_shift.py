"""Compare per-submission host CPU (c_rt) between the r3 segment-one product and the r4 drift-control arms.

Reads point lines only (candidate=scan_checksum_only, threshold_kind=attack_ledger, clock given),
keys by (index, form, wait, gap, fraction, pool, batch). Prints the median over keys of
median(c_rt rounds of r4 arm) / median(c_rt rounds of r3), per arm, card, clock and form class.
"""
import statistics, sys

def fields(line):
    out = {}
    for tok in line.split():
        if '=' in tok:
            k, v = tok.split('=', 1)
            out[k] = v
    return out

def load(path, name, extra=None):
    rows = {}
    with open(path) as f:
        for line in f:
            if ('name=%s ' % name) not in line:
                continue
            v = fields(line)
            if v.get('candidate') != 'scan_checksum_only' or v.get('threshold_kind') != 'attack_ledger':
                continue
            key = (v.get('arm', 'r3'), v['index'], v['clock'], v['form'], v['wait'], v['gap_milliseconds'], v['fraction'], v['pool_data_units'], v['batch_units'])
            per = [float(x) for x in v['host_cpu_per_submission_milliseconds_per_round'].split(',') if x not in ('not_run', '')]
            rows[key] = statistics.median(per) if per else None
    return rows

r3 = load(sys.argv[1], 'host_cpu_per_submission_point')
r4 = load(sys.argv[2], 'drift_control_point')
groups = {}
for key, val in r4.items():
    arm = key[0]
    k3 = ('r3',) + key[1:]
    if k3 not in r3 or r3[k3] is None or val is None:
        continue
    form_class = 'today' if key[3] == 'today_single_kernel' else 'gpu'
    g = (arm, key[1], key[2], form_class)
    groups.setdefault(g, []).append(val / r3[k3])
for g in sorted(groups):
    xs = sorted(groups[g])
    print('name=crt_shift arm=%s index=%s clock=%s forms=%s keys=%d median_r4_over_r3=%.3f min=%.3f max=%.3f below_one=%d' % (g + (len(xs), statistics.median(xs), xs[0], xs[-1], sum(1 for x in xs if x < 1))))
