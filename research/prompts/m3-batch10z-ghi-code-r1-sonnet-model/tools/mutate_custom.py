#!/usr/bin/env python3
"""Run own mutations (json list file) on copy: [{name,file,old,new,cmd}]"""
import subprocess, sys, os, re, json
COPY, TARGET, LIST = sys.argv[1:4]
SEL = sys.argv[4] if len(sys.argv) > 4 else None
muts = json.load(open(LIST, encoding='utf-8'))
for m in muts:
    if SEL and SEL not in m['name']: continue
    f = os.path.join(COPY, m['file'])
    src = open(f, encoding='utf-8').read()
    n = src.count(m['old'])
    if n != 1:
        print('SKIP', m['name'], 'old occurs', n, flush=True); continue
    open(f, 'w', encoding='utf-8').write(src.replace(m['old'], m['new']))
    try:
        args = ['bash','/home/fy5090/code/singlefs/scripts/hosts/capped.sh','15','bash',
                '/home/fy5090/code/singlefs/scripts/hosts/run-with-memory-cap.sh','8G','nice','-n','19',
                'cargo','test','--offline'] + m['cmd'].split()
        env = dict(os.environ, CARGO_TARGET_DIR=TARGET)
        p = subprocess.run(args, cwd=COPY, env=env, capture_output=True, text=True, timeout=1500)
        out = p.stdout + p.stderr
        res = re.findall(r'test result: .*', out)
        failed = re.findall(r'^test (\S+) \.\.\. FAILED', out, re.M)
        comp = 'error: could not compile' in out
        v = 'COMPILE-ERROR' if comp else ('RED(caught)' if p.returncode != 0 else 'GREEN(not caught)')
        print(v, '|', m['name'], '| rc', p.returncode, '|', res[-1] if res else '', '| failed:', failed, flush=True)
    finally:
        open(f, 'w', encoding='utf-8').write(src)
