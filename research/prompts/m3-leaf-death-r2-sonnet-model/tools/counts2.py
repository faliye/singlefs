import glob, collections, sys
def parse(line):
    d = {}
    for t in line.split()[1:]:
        if '=' in t:
            k,v = t.split('=',1); d[k]=v
    return d
files = sys.argv[1:]
acc = collections.defaultdict(lambda: collections.Counter())
mx = collections.defaultdict(lambda: collections.Counter())
for f in files:
    for line in open(f):
        if 'name=leaf_counts' not in line: continue
        d = parse(line)
        k = (d['geometry'], d['policy'])
        a = acc[k]; a['runs'] += 1
        for key in ('publishes','merges','borrows','borrowed_records','zero_versions','oneshot_drops','record_deaths','removal_publishes','containers_rewritten_in_removals','versions_with_an_empty_container','containers_final','empty_containers_final','retirements'):
            a[key] += int(d[key])
        for key in ('containers_peak','empty_containers_peak'):
            mx[k][key] = max(mx[k][key], int(d[key]))
for k in sorted(acc):
    a = acc[k]; m = mx[k]
    print(' '.join(k), ' '.join(f'{x}={a[x]}' for x in ('runs','publishes','merges','borrows','borrowed_records','zero_versions','retirements','versions_with_an_empty_container','containers_final','empty_containers_final')), 'containers_peak_max=%d empty_peak_max=%d' % (m['containers_peak'], m['empty_containers_peak']))
