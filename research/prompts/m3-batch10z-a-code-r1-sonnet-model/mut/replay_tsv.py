# 把 mutations.tsv 里指定行（行号）的 harness 档变异在 tree-mut 副本里逐条改、跑点名的测试、还原；红了写「抓到」。
import subprocess, sys, os
BASE = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-a-code-r1-sonnet')
R = '/home/fy5090/code/singlefs/scripts'
rows = open(f'{BASE}/tree/crates/mutations.tsv').read().split('\n')
for n in map(int, sys.argv[1:]):
    f = rows[n - 1].split('\t')
    name, rel, old, new, args = f[0], f[1], f[2].replace('\\n', '\n'), f[3].replace('\\n', '\n'), f[4]
    src = f'{BASE}/tree/{rel}'
    dst = f'{BASE}/tree-mut/{rel}'
    orig = open(src).read()
    assert orig.count(old) == 1, (n, orig.count(old))
    open(dst, 'w').write(orig.replace(old, new))
    cmd = f'cd {BASE}/tree-mut && CARGO_TARGET_DIR={BASE}/target-mut nice -n 19 bash {R}/run-with-memory-cap.sh 12G bash {R}/capped.sh 15 cargo test --offline {args}'
    p = subprocess.run(['bash', '-c', cmd], capture_output=True, text=True)
    open(dst, 'w').write(orig)
    out = p.stdout + p.stderr
    failed = [l.split()[1].split('::')[-1] for l in out.splitlines() if l.startswith('test ') and l.endswith('FAILED')]
    verdict = 'caught' if p.returncode != 0 and failed else ('compile-error' if 'error[' in out else 'SURVIVED')
    print(f'row {n} {verdict} exit={p.returncode} red={len(failed)} :: {name}', flush=True)
