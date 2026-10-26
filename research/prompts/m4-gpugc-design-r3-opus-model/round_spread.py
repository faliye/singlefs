"""Five-round spread (max / min) and level of c_rt per run and card, candidate A, machine_scaled, thread clock.

argv: r3 segment-one product, r4 drift-control product.
"""
import collections, statistics, sys

def lines(path, name):
    with open(path) as f:
        for l in f:
            if ('name=%s ' % name) in l:
                yield dict(t.split('=', 1) for t in l.split() if '=' in t)

out = collections.defaultdict(list)
for v in lines(sys.argv[2], 'drift_control_point'):
    if v['candidate'] == 'scan_checksum_only' and v['threshold_kind'] == 'machine_scaled' and v['clock'] == 'thread':
        x = [float(a) for a in v['host_cpu_per_submission_milliseconds_per_round'].split(',')]
        out[(v['arm'], v['index'])].append((max(x) / min(x), statistics.median(x)))
for v in lines(sys.argv[1], 'host_cpu_per_submission_point'):
    if v['candidate'] == 'scan_checksum_only' and v['threshold_kind'] == 'machine_scaled' and v['clock'] == 'thread':
        x = [float(a) for a in v['host_cpu_per_submission_milliseconds_per_round'].split(',')]
        out[('r3', v['index'])].append((max(x) / min(x), statistics.median(x)))
for k in sorted(out):
    s = [a for a, b in out[k]]
    m = [b for a, b in out[k]]
    print('name=round_spread run=%s index=%s lines=%d max_over_min_median=%.3f max_over_min_max=%.3f c_rt_median_of_point_medians_ms=%.4f' % (k[0], k[1], len(s), statistics.median(s), max(s), statistics.median(m)))
