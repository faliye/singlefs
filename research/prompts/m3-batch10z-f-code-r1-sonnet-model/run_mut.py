import json, os, shutil, subprocess, sys, time
W = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-f-code-r1-sonnet')
REPO = '/home/fy5090/code/singlefs'
worker, nworkers, lo, hi = int(sys.argv[1]), int(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4])
rows = json.load(open(W + '/' + os.environ.get('ROWS','rows.json')))
tree = f'{W}/copy-w{worker}'
tgt = f'{W}/target-w{worker}'
if not os.path.isdir(tree):
    shutil.copytree(W + '/copy-a', tree, symlinks=True)
env = dict(os.environ, CARGO_TARGET_DIR=tgt)
def run(args, timeout=600):
    cmd = ['nice', '-n', '19', 'bash', REPO + '/research/scripts/run-with-memory-cap.sh', '1G',
           'bash', REPO + '/research/scripts/capped.sh', '2', 'cargo', 'test', '--offline'] + args
    t = time.time()
    p = subprocess.run(cmd, cwd=tree, env=env, capture_output=True, text=True, timeout=timeout)
    return p.returncode, p.stdout + p.stderr, time.time() - t
out = open(f'{W}/{os.environ.get('LOGP','mut')}-w{worker}.log', 'a')
for i, r in enumerate(rows):
    if i < lo or i >= hi or i % nworkers != worker: continue
    if 'singlefs-harness' not in r['cmd']:
        out.write(f'ROW {i} SKIP(not harness) {r["name"]}\n'); out.flush(); continue
    path = f'{tree}/{r["file"]}'
    src = open(path, encoding='utf-8').read()
    assert src.count(r['anchor']) == 1, (i, src.count(r['anchor']))
    mutated = src.replace(r['anchor'], r['repl'])
    open(path + '.mut', 'w', encoding='utf-8').write(mutated)
    os.replace(path + '.mut', path)
    try:
        args = r['cmd'].split()
        rc, text, dt = run(args)
    finally:
        open(path + '.orig', 'w', encoding='utf-8').write(src)
        os.replace(path + '.orig', path)
    lines = [l for l in text.split('\n') if l.startswith('test result') or ' FAILED' in l or 'error[' in l or l.startswith('error')]
    verdict = 'RED(caught)' if rc != 0 and any('FAILED' in l or 'failed' in l for l in lines) else ('COMPILE-ERR/OTHER' if rc != 0 else 'GREEN(survived)')
    out.write(f'ROW {i} {verdict} rc={rc} {dt:.0f}s kind={r["kind"]} {r["name"]}\n')
    for l in lines[:6]: out.write('   ' + l + '\n')
    out.flush()
