"""N5 探针输出的摘要：每个 K 一行（卸载崩点个数、暖机推两次的崩点、坏态数、新用例那一判会红的崩点），另列有坏态的崩点。"""
import re, sys
from collections import defaultdict
for path in sys.argv[1:]:
    variant = path.split('n5-probe-')[1].rsplit('-k', 1)[0]
    ks = {}
    states = defaultdict(list)
    for line in open(path, encoding='utf-8'):
        if line.startswith('N5_K '):
            d = dict(re.findall(r'(\w+)=(\S+)', line))
            ks[int(d['k'])] = d
        elif line.startswith('N5_STATE '):
            d = dict(re.findall(r'(\w+)=(\S*)', line.split(' first_bad=')[0]))
            states[int(d['k'])].append(d)
    for k in sorted(ks):
        rows = states[k]
        bad = [r for r in rows if int(r.get('bad_states', 0)) > 0]
        twice = [int(r['c']) for r in rows if r.get('warm_ups') == '2']
        caught = [int(r['c']) for r in rows if int(r.get('taken_into_released_by_current', 0)) > 0]
        refused = [r['c'] for r in rows if r.get('remount', '').startswith('refused')]
        print(f"N5SUM variant={variant} k={k} current_txg={ks[k].get('current_txg')} mod3={ks[k].get('current_txg_mod3')} "
              f"crash_points={len(rows)} refused={len(refused)} warm_up_twice_c={min(twice) if twice else '-'}..{max(twice) if twice else '-'} "
              f"remount_crash_states={sum(int(r.get('remount_crash_states', 0)) for r in rows)} bad_states={sum(int(r.get('bad_states', 0)) for r in rows)} "
              f"crash_points_with_bad={','.join(r['c'] for r in bad) or '-'} test_check_nonzero_c={','.join(map(str, caught)) or '-'}")
