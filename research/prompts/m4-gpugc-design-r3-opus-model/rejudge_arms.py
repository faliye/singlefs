"""Independent re-judgement of the r4 drift-control point lines against the r4 rejudge thresholds.

Inputs: the r4 drift-control product and the r4 baseline-rejudge product (only its baseline_threshold
and baseline_rejudge_summary lines are read). Does not import the repository's rejudge script.

Rules (r3 registration R1-R3, copied from the product semantics):
  per round: c_rt < threshold -> not_losing (scan_checksum_and_choice: not_losing_lower_bound), else losing;
  five rounds all equal -> that judgement, otherwise unstable; a missing round -> rounds_insufficient;
  a line whose own judgement starts with void_ keeps it.

Outputs (stdout):
  name=positive_control ...  every controlled-arm summary equals the product's baseline_rejudge_summary
  name=arm_summary ...       per arm, writing, threshold source, candidate, card, clock, form class
  name=gap_summary ...       the same split by gap
  name=round_robin ...       the controlled arm, both cards averaged round by round
Optional argv[3] = 'plant' plants a fault (thresholds read 1% high) so the positive control must fail.
"""
import collections, statistics, sys

LOWER = 'scan_checksum_and_choice'
KEY = ('fraction', 'pool_data_units', 'batch_units', 'candidate', 'index', 'form', 'wait', 'gap_milliseconds', 'clock', 'threshold_kind')

def fields(line):
    out = {}
    for tok in line.split():
        if '=' in tok:
            k, v = tok.split('=', 1)
            out[k] = v
    return out

def num(t):
    try:
        return float(t)
    except ValueError:
        return None

PLANT = len(sys.argv) > 3 and sys.argv[3] == 'plant'

def one(value, threshold, candidate):
    below = value < threshold * 1.01 if PLANT else value < threshold
    if below:
        return 'not_losing_lower_bound' if candidate == LOWER else 'not_losing'
    return 'losing'

def judge(values, threshold, candidate, own):
    if own.startswith('void_'):
        return own
    if threshold is None:
        return 'not_run'
    per = [None if v is None else one(v, threshold, candidate) for v in values]
    if len(per) < 5 or any(p is None for p in per):
        return 'rounds_insufficient'
    return per[0] if all(p == per[0] for p in per) else 'unstable'

drift_path, rejudge_path = sys.argv[1], sys.argv[2]
points = []
with open(drift_path) as f:
    for line in f:
        if 'name=drift_control_point ' in line:
            points.append(fields(line))

thresholds = {}
product_summary = {}
with open(rejudge_path) as f:
    for line in f:
        if 'name=baseline_threshold ' in line:
            v = fields(line)
            if v['crt_source'] != 'r4_drift_control_controlled':
                continue
            thresholds[(v['writing'], v['threshold_source'], v['threshold_kind'], v['fraction'], v['pool_data_units'], v['batch_units'], v['candidate'])] = (
                None if v['status'] != 'status=measured' and not v.get('status', '').startswith('measured') and v['threshold_milliseconds'] == 'none' else num(v['threshold_milliseconds']))
        elif 'name=baseline_rejudge_summary ' in line:
            v = fields(line)
            if v['crt_source'] != 'r4_drift_control_controlled':
                continue
            k = (v['writing'], v['threshold_source'], v['candidate'], v['index'], v['threshold_kind'], v['clock'])
            product_summary[k] = tuple(int(v[c]) for c in ('lines', 'not_losing', 'not_losing_lower_bound', 'losing', 'unstable'))

writings = sorted({k[0] for k in thresholds})
sources = sorted({k[1] for k in thresholds})
judged = []  # (arm, writing, source, point fields, judgement)
for p in points:
    values = [num(x) for x in p['host_cpu_per_submission_milliseconds_per_round'].split(',')]
    for w in writings:
        for s in sources:
            t = thresholds.get((w, s, p['threshold_kind'], p['fraction'], p['pool_data_units'], p['batch_units'], p['candidate']))
            judged.append((p['arm'], w, s, p, judge(values, t, p['candidate'], p.get('judgement', ''))))

def tally(rows):
    c = collections.Counter(j for j in rows)
    return (sum(c.values()), c['not_losing'], c['not_losing_lower_bound'], c['losing'], c['unstable'])

mine = collections.defaultdict(list)
for arm, w, s, p, j in judged:
    if arm == 'controlled':
        mine[(w, s, p['candidate'], p['index'], p['threshold_kind'], p['clock'])].append(j)
mismatch = [k for k in product_summary if tally(mine.get(k, [])) != product_summary[k]]
print('name=positive_control item=controlled_arm_equals_the_product_summaries summaries=%d mismatches=%d holds=%s' % (len(product_summary), len(mismatch), str(not mismatch).lower()))

groups = collections.defaultdict(list)
gaps = collections.defaultdict(list)
for arm, w, s, p, j in judged:
    fc = 'today' if p['form'] == 'today_single_kernel' else 'gpu'
    groups[(arm, w, s, p['candidate'], p['index'], p['threshold_kind'], p['clock'], fc)].append(j)
    gaps[(arm, w, s, p['candidate'], p['index'], p['threshold_kind'], p['clock'], fc, p['gap_milliseconds'])].append(j)
for k in sorted(groups):
    n, nl, nlb, l, u = tally(groups[k])
    print('name=arm_summary arm=%s writing=%s threshold_source=%s candidate=%s index=%s threshold_kind=%s clock=%s forms=%s lines=%d not_losing=%d not_losing_lower_bound=%d losing=%d unstable=%d' % (k + (n, nl, nlb, l, u)))
for k in sorted(gaps):
    n, nl, nlb, l, u = tally(gaps[k])
    print('name=gap_summary arm=%s writing=%s threshold_source=%s candidate=%s index=%s threshold_kind=%s clock=%s forms=%s gap_milliseconds=%s lines=%d not_losing=%d not_losing_lower_bound=%d losing=%d unstable=%d' % (k + (n, nl, nlb, l, u)))

# round robin: controlled arm, same key on both cards, per-round mean of the two cards' c_rt
by = {}
for p in points:
    if p['arm'] != 'controlled':
        continue
    by[(p['index'],) + tuple(p[x] for x in KEY if x != 'index')] = p
rr = collections.Counter()
for k, p0 in by.items():
    if k[0] != '0':
        continue
    p1 = by.get(('1',) + k[1:])
    if p1 is None or p0['clock'] != 'thread' or p0['threshold_kind'] != 'machine_scaled' or p0['form'] == 'today_single_kernel':
        continue
    v0 = [num(x) for x in p0['host_cpu_per_submission_milliseconds_per_round'].split(',')]
    v1 = [num(x) for x in p1['host_cpu_per_submission_milliseconds_per_round'].split(',')]
    vm = [None if a is None or b is None else (a + b) / 2 for a, b in zip(v0, v1)]
    for w in ('today', 'both_three_way_without_build'):
        s = 'direct_from_the_r4_product'
        t = thresholds.get((w, s, p0['threshold_kind'], p0['fraction'], p0['pool_data_units'], p0['batch_units'], p0['candidate']))
        rr[(w, p0['candidate'], judge(v0, t, p0['candidate'], ''), judge(vm, t, p0['candidate'], ''))] += 1
for k in sorted(rr):
    print('name=round_robin arm=controlled writing=%s candidate=%s judgement_card0_alone=%s judgement_round_robin=%s cells=%d' % (k + (rr[k],)))
