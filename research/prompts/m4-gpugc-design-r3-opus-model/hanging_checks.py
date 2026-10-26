"""Numbers for the hanging cells (G4 N, G6 fix) and the verdict numbers of rounds one and two, recounted from the products.

argv: repo root. Reads research/results/ products and the round-one attack model output g6.out. Prints name=... lines.
"""
import collections, os, statistics, sys

root = sys.argv[1]
res = lambda name: os.path.join(root, 'research/results', name)
DRIFT = res('e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out')
WRIT = res('e185-gc-batch-wall-clock-split-cpu-writings-r4-2026-10-24.out')
R3 = res('e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out')
G6 = os.path.join(root, 'research/prompts/m4-gpugc-design-r1-opus-model/out/g6.out')

def fields(line):
    out = {}
    for tok in line.split():
        if '=' in tok:
            k, v = tok.split('=', 1)
            out[k] = v
    return out

def rows(path, name):
    with open(path) as f:
        return [fields(l) for l in f if ('name=%s ' % name) in l]

# G4 N: every timed submission of a valid cell compares the two kernels with the CPU on every unit.
acc = collections.defaultdict(lambda: [0, 0, 0, 0, 0])
for r in rows(DRIFT, 'submission_cell'):
    kernels = 'one_kernel_today' if r['form'] == 'today_single_kernel' else 'two_kernels'
    a = acc[(r['arm'], r['index'], kernels)]
    if r['invalid_reason'] != 'none':
        a[4] += 1
        continue
    timed = 100 if r['gap_milliseconds'] == '0' else 50
    a[0] += 1
    a[1] += int(r['uncertain_entries'])
    a[2] += r['results_agree_with_the_cpu'] != 'true'
    a[3] += int(r['units']) * timed
for k in sorted(acc):
    v = acc[k]
    print('name=n_count arm=%s index=%s kernels=%s valid_cells=%d uncertain_entries=%d cells_disagreeing_with_the_cpu=%d unit_comparisons_in_timed_submissions=%d invalid_cells=%d' % (k + tuple(v)))
for kernels in ('two_kernels', 'one_kernel_today'):
    tot = [sum(acc[k][i] for k in acc if k[2] == kernels) for i in range(5)]
    print('name=n_count arm=all index=all kernels=%s valid_cells=%d uncertain_entries=%d cells_disagreeing_with_the_cpu=%d unit_comparisons_in_timed_submissions=%d invalid_cells=%d' % ((kernels,) + tuple(tot)))
units = collections.Counter(r['units'] for r in rows(DRIFT, 'submission_cell'))
print('name=n_units ' + ' '.join('units_%s=%d' % (u, units[u]) for u in sorted(units, key=int)))

# G6: one batch's host CPU (crates call, today's writing) and the same call with the three CRC stages and the scan
# swapped for the three-way hardware CRC and the identity lookup (index build not counted).
stage = {}
for r in rows(WRIT, 'cpu_writing_stage'):
    stage[(r['fraction'], r['pool_data_units'], r['batch_units'], r['round'], r['call_index'])] = r
out = collections.defaultdict(lambda: ([], []))
for c in rows(WRIT, 'call'):
    if c['kind'] == 'issue':
        continue
    s = stage[(c['fraction'], c['pool_data_units'], c['batch_units'], c['round'], c['call_index'])]
    today = float(c['cpu_nanoseconds'])
    saved = (float(s['crc_today_cpu_nanoseconds']) - float(s['crc_hardware_three_way_cpu_nanoseconds'])
             + float(s['release_check_crc_today_cpu_nanoseconds']) - float(s['release_check_crc_hardware_three_way_cpu_nanoseconds'])
             + float(s['scan_today_cpu_nanoseconds']) - float(s['scan_identity_index_lookup_cpu_nanoseconds']))
    t, w = out[(c['fraction'], c['pool_data_units'])]
    t.append(today / 1e6)
    w.append((today - saved) / 1e6)
for k in sorted(out, key=lambda k: (k[0], int(k[1]))):
    t, w = out[k]
    print('name=batch_cpu fraction=%s pool_data_units=%s calls=%d today_ms_min=%.1f today_ms_max=%.1f both_three_way_without_build_ms_min=%.1f both_three_way_without_build_ms_max=%.1f largest_share_saved=%.4f' % (
        k + (len(t), min(t), max(t), min(w), max(w), max(1 - b / a for a, b in zip(t, w)))))
notes = collections.Counter(r.get('snapshot_branch_note', 'absent') for r in rows(WRIT, 'stage'))
print('name=snapshot_note ' + ' '.join('%s=%d' % kv for kv in sorted(notes.items())))

# Round two: one-submission wall clock (defence G1-3), at most 32 units, shared input, per (card, wait, gap, units) median.
w = collections.defaultdict(list)
for r in rows(R3, 'submission_cell'):
    if r['form'] == 'shared' and int(r['units']) <= 32 and r['invalid_reason'] == 'none':
        w[(r['index'], r['wait'], r['gap_milliseconds'], r['units'])].append(float(r['wall_median_nanoseconds']) / 1e6)
meds = [statistics.median(v) for v in w.values()]
print('name=r2_check item=one_submission_wall cells=%d min=%.2f max=%.2f' % (len(meds), min(meds), max(meds)))
for r in rows(R3, 'calibration_baseline'):
    if r['position'] == 'end':
        print('name=r2_check item=calibration_end_over_start ratio=%s faster_by=%.3f' % (r['ratio_of_the_end_to_the_start'], 1 - float(r['ratio_of_the_end_to_the_start'])))
fits = collections.Counter((r['index'], r['gap_milliseconds'], r['fit_not_linear']) for r in rows(R3, 'fixed_and_per_byte_fit'))
for k in sorted(fits):
    print('name=r2_check item=r3_fit index=%s gap_milliseconds=%s fit_not_linear=%s lines=%d' % (k + (fits[k],)))

# Round one: real-disk arm batch wall clock in the round-one attack model g6 and the round-two defence share 4.5%-118%.
g6 = rows(G6, 'g6_batch')
for arm in ('memory', 'real_disk'):
    v = [float(r['wall_ms_median']) for r in g6 if r['arm'] == arm]
    print('name=r1_check item=g6_batch_wall arm=%s lines=%d wall_ms_median_min=%.3f max=%.3f' % (arm, len(v), min(v), max(v)))
mem1024 = [float(r['wall_ms_median']) for r in g6 if r['arm'] == 'memory' and r['P'] == '1024' and r['f'] == '0.25' and r['K'] == '4'][0]
disk8192 = [float(r['wall_ms_median']) for r in g6 if r['arm'] == 'real_disk' and r['P'] == '8192' and r['f'] == '0.25' and r['K'] == '8'][0]
print('name=r2_check item=g8_share low=%.4f high=%.4f from_increment_ms=5.50,14.76 over_batch_ms=%.3f,%.3f' % (5.50 / disk8192, 14.76 / mem1024, disk8192, mem1024))

# M8: process clock against thread clock on the same key (process lines exist only where J5 says the thread clock under-reads by > 10%).
pair = collections.defaultdict(dict)
for r in rows(DRIFT, 'drift_control_point'):
    if r['candidate'] != 'scan_checksum_only' or r['threshold_kind'] != 'machine_scaled':
        continue
    key = (r['arm'], r['index'], r['fraction'], r['pool_data_units'], r['batch_units'], r['form'], r['wait'], r['gap_milliseconds'])
    v = [float(x) for x in r['host_cpu_per_submission_milliseconds_per_round'].split(',')]
    pair[key][r['clock']] = statistics.median(v)
ratio = collections.defaultdict(list)
total = collections.Counter()
for key, d in pair.items():
    total[(key[0], key[1])] += 1
    if 'process' in d and 'thread' in d:
        ratio[(key[0], key[1])].append(d['process'] / d['thread'])
for k in sorted(total):
    v = ratio[k]
    print('name=clock_pair arm=%s index=%s keys=%d keys_with_a_process_line=%d process_over_thread_median=%s min=%s max=%s' % (
        k + (total[k], len(v), '%.3f' % statistics.median(v) if v else 'none', '%.3f' % min(v) if v else 'none', '%.3f' % max(v) if v else 'none')))
