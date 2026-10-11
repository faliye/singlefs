import os, re, shutil, subprocess
W = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-pqrs-code-r1-sonnet')
T = W + '/tree-m'
R = '/home/fy5090/code/singlefs/scripts'
lines = open('/home/fy5090/code/singlefs/research/prompts/_m3-batch10z-pqrs-code-r1-diff.md').read().split('\n')
f = lines[1523].split('\t')
old, new = f[2].replace('\\n', '\n'), f[3].replace('\\n', '\n')
path = T + '/' + f[1]
src = open(path).read()
assert src.count(old) == 1
shutil.copy(path, path + '.orig')
try:
    open(path, 'w').write(src.replace(old, new, 1))
    env = dict(os.environ, CARGO_TARGET_DIR=W + '/target-m')
    r = subprocess.run(['nice', '-n', '19', 'bash', R + '/run-with-memory-cap.sh', '6G', 'bash', R + '/capped.sh', '8', 'cargo', 'test', '--release', '--offline', '-p', 'singlefs-harness', '--test', 'raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table', '--', '--include-ignored'], cwd=T, env=env, capture_output=True, text=True)
    out = r.stdout + r.stderr
    print('rc', r.returncode, re.findall(r'^test result: .*', out, re.M)[-1:], re.findall(r'^test (\S+) \.\.\. FAILED', out, re.M))
finally:
    shutil.move(path + '.orig', path)
