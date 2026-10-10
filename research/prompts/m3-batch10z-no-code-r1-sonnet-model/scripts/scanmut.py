#!/usr/bin/env python3
import json, os, subprocess
W = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-no-code-r1-sonnet')
T = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-no-code-r1/tree/')
GPU = 'crates/singlefs-checker-tier/src/gpu_unit_checks.rs'
CR = 'crates/singlefs-checker-tier/src/crash_amplification.rs'
def run(mode, path):
    return subprocess.run(['bash', W + '/scripts/scan.sh', mode, path], capture_output=True, text=True).stdout.strip()
rows = []
for i, r in enumerate(json.load(open(W + '/gpu/repo-rows.json'))):
    if r['file'].endswith('gpu_unit_checks.rs'): rows.append(('repo%d' % i + ' ' + r['name'][-40:], GPU, r['old'], r['new']))
    if r['file'].endswith('crash_amplification.rs'): rows.append(('repo%d' % i + ' ' + r['name'][-40:], CR, r['old'], r['new']))
extra = json.load(open(W + '/gpu/scan-extra.json')) if os.path.exists(W + '/gpu/scan-extra.json') else []
for e in extra: rows.append((e['id'], e['file'], e['old'], e['new']))
print('baseline gpu  :', run('gpu', T + GPU)); print('baseline crash:', run('crash', T + CR))
for name, file, old, new in rows:
    src = open(T + file, encoding='utf-8').read()
    n = src.count(old)
    if n != 1:
        print(name[:70], '| OLD-NOT-ONCE', n); continue
    p = W + '/scanmut/m.rs'; open(p, 'w', encoding='utf-8').write(src.replace(old, new))
    print(name[:80], '|', 'gpu' if file == GPU else 'crash', '|', run('gpu' if file == GPU else 'crash', p)[:120])
