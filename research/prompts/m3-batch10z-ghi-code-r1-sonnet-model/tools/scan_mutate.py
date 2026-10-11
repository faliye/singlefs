#!/usr/bin/env python3
"""Run the scan-model tests against mutated copies of checker-tier sources. rows: from diff TSV (checker-tier files) + extras json."""
import subprocess, sys, os, re, json, tempfile
S = '/home/fy5090/.local/share/singlefs/work/m3-batch10z-ghi-code-r1-sonnet'
T = '/home/fy5090/.local/share/singlefs/work/m3-batch10z-ghi-code-r1/tree/crates/singlefs-checker-tier/src/'
DIFF = '/home/fy5090/code/singlefs/research/prompts/_m3-batch10z-ghi-code-r1-diff.md'
rows = []
for line in open(DIFF, encoding='utf-8'):
    line = line.rstrip('\n')
    if line.count('\t') == 5 and line.startswith('第10') and ('checker-tier/src/crash_amplification.rs' in line or 'checker-tier/src/gpu_unit_checks.rs' in line):
        n, f, o, nw, c, t = line.split('\t'); rows.append(dict(name=n, file=f.split('/')[-1], old=o.replace('\\n','\n'), new=nw.replace('\\n','\n'), test=t))
extras = json.load(open(sys.argv[1], encoding='utf-8')) if len(sys.argv) > 1 else []
only = sys.argv[2] if len(sys.argv) > 2 else None
allrows = [dict(r, src='diff') for r in rows] + [dict(r, src='own') for r in extras]
for r in allrows:
    if only and only not in r['name']: continue
    path = T + r['file']
    src = open(path, encoding='utf-8').read()
    if src.count(r['old']) != 1:
        print('SKIP', r['src'], r['name'][:70], 'occurs', src.count(r['old'])); continue
    with tempfile.TemporaryDirectory(dir=S) as d:
        mp = os.path.join(d, r['file'])
        open(mp, 'w', encoding='utf-8').write(src.replace(r['old'], r['new']))
        env = dict(os.environ, CARGO_TARGET_DIR=S + '/target-scan',
                   CA_SRC=mp if r['file'] == 'crash_amplification.rs' else T + 'crash_amplification.rs',
                   GU_SRC=mp if r['file'] == 'gpu_unit_checks.rs' else T + 'gpu_unit_checks.rs')
        p = subprocess.run(['bash','/home/fy5090/code/singlefs/scripts/capped.sh','15','bash','/home/fy5090/code/singlefs/scripts/run-with-memory-cap.sh','4G','nice','-n','19','cargo','test','--offline'], cwd=S + '/scan-model', env=env, capture_output=True, text=True)
        out = p.stdout + p.stderr
        failed = re.findall(r'^test (\S+) \.\.\. FAILED', out, re.M)
        covers = r['test'] in ' '.join(failed) if r.get('test') else None
        print(('RED(caught)' if p.returncode else 'GREEN(not caught)'), '|', r['src'], '|', r['name'][:75], '| failed:', [x.split('::')[-1][:50] for x in failed], flush=True)
