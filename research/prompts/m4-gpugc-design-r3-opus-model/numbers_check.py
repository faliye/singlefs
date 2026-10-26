"""Recount the numbers that the round-two verdict and the round-three body report, from the products.

argv: repo root. Reads only research/results/ products. Prints one name=check line per number.
"""
import collections, os, statistics, sys

root = sys.argv[1]
res = lambda name: os.path.join(root, 'research/results', name)
R3 = res('e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out')
FS = res('e185-gc-batch-wall-clock-split-function-split-r3-2026-10-24.out')
SS = res('e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out')
GS = res('e185-gc-batch-wall-clock-split-gpu-submission-r1-2026-10-22.out')

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

pts = rows(R3, 'host_cpu_per_submission_point')
c = collections.Counter(p['judgement'] for p in pts)
print('name=check item=r3_point_lines lines=%d losing=%d not_losing=%d not_losing_lower_bound=%d unstable=%d' % (len(pts), c['losing'], c['not_losing'], c['not_losing_lower_bound'], c['unstable']))

by = {}
for p in pts:
    key = tuple(p[k] for k in ('fraction', 'pool_data_units', 'batch_units', 'index', 'form', 'wait', 'gap_milliseconds', 'clock', 'threshold_kind'))
    by.setdefault(key, {})[p['candidate']] = p['host_cpu_per_submission_milliseconds_per_round']
groups = [v for v in by.values() if 'every_stage' in v and 'scan_checksum_only' in v]
same = sum(1 for v in groups if v['every_stage'] == v['scan_checksum_only'])
print('name=check item=c_and_a_same_c_rt groups=%d identical=%d' % (len(groups), same))

for card in ('0', '1'):
    ratios = []
    for p in pts:
        if p['candidate'] == 'scan_checksum_only' and p['threshold_kind'] == 'machine_scaled' and p['clock'] == 'thread' and p['index'] == card:
            v = [float(x) for x in p['host_cpu_per_submission_milliseconds_per_round'].split(',')]
            ratios.append(v[4] / v[0])
    print('name=check item=r3_round_five_over_round_one index=%s lines=%d median=%.3f' % (card, len(ratios), statistics.median(ratios)))

for s in rows(FS, 'function_split_summary'):
    parts = sum(float(v) for k, v in s.items() if k.endswith('_share_median') and k not in ('call_share_median', 'remainder_share_median'))
    print('name=check item=function_split_share_sum fraction=%s pool=%s scope=%s sum=%.3f remainder=%s' % (s['fraction'], s['pool_data_units'], s['scope'], parts, s['remainder_share_median']))

ms = rows(R3, 'merging_shape')
mc = collections.Counter(m['category'] for m in ms)
print('name=check item=r3_merging_shape lines=%d merge_across_intents=%d' % (len(ms), mc['merge_across_intents']))

cells = rows(R3, 'submission_cell')
crc = {r['units']: float(r['thread_cpu_median_nanoseconds']) / 1e6 for r in rows(R3, 'cpu_crc_of_the_same_bytes')}
walls = collections.defaultdict(list)
for r in cells:
    if r['form'] == 'shared' and r.get('invalid_reason') == 'none':
        walls[(r['index'], r['units'])].append(float(r['wall_median_nanoseconds']) / 1e6)
for (card, units), seq in (( ('0', '8'), 1), (('1', '24'), 2)):
    pass
def per_key_medians(card, units):
    acc = collections.defaultdict(list)
    for r in cells:
        if r['form'] == 'shared' and r['index'] == card and r['units'] == units and r.get('invalid_reason') == 'none':
            acc[(r['wait'], r['gap_milliseconds'])].append(float(r['wall_median_nanoseconds']) / 1e6)
    return [statistics.median(v) for v in acc.values()]
w8 = per_key_medians('0', '8'); w24 = per_key_medians('1', '24')
print('name=check item=g8_foreground_increment card0_one_submission_of_8_units_min=%.2f max=%.2f card1_two_submissions_of_24_units_min=%.2f max=%.2f' % (
    min(w8) - crc['8'], max(w8) - crc['8'], 2 * (min(w24) - crc['24']), 2 * (max(w24) - crc['24'])))
gw = [float(r['wall_median_nanoseconds']) / 1e6 for r in cells if r['form'] in ('shared', 'split') and r.get('invalid_reason') == 'none']
print('name=check item=r3_one_submission_wall_median_range min=%.2f max=%.2f cells=%d' % (min(gw), max(gw), len(gw)))

with open(SS) as f:
    l989 = f.read().splitlines()[988]
v = fields(l989)
share = (float(v['crc_cpu_nanoseconds']) + float(v['release_check_crc_cpu_nanoseconds']) + float(v['scan_increment_cpu_nanoseconds'])) / float(v['call_cpu_nanoseconds'])
print('name=check item=r1_verdict_stage_share line=989 share=%.6f' % share)
with open(GS) as f:
    gl = f.read().splitlines()
print('name=check item=r1_verdict_intercepts line54=%s line136=%s' % (fields(gl[53]).get('intercept_seconds'), fields(gl[135]).get('intercept_seconds')))
