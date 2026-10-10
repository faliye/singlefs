#!/usr/bin/env python3
"""rows.py <起行> <止行>：把改动全文里 mutations 行（制表符分六列：名、文件、旧串、新串、命令、用例）逐行套到 tree-m、编、跑那一行点名的命令（release），报红几条；还原。"""
import os, re, shutil, subprocess, sys, time
W = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-tu-code-r1-sonnet')
T = W + '/' + os.environ.get('TREE','tree-m')
R = '/home/fy5090/code/singlefs/research/scripts'
lines = open('/home/fy5090/code/singlefs/research/prompts/_m3-batch10z-tu-code-r1-diff.md').read().split('\n')
a, b = int(sys.argv[1]), int(sys.argv[2])
for ln in range(a, b + 1):
    f = lines[ln - 1].split('\t')
    name, file, old, new, cmd, test = f[0], f[1], f[2].replace('\\n', '\n'), f[3].replace('\\n', '\n'), f[4], f[5] if len(f) > 5 else ''
    path = os.path.join(T, file)
    src = open(path).read()
    t0 = time.time()
    if src.count(old) != 1:
        print(f'行{ln} {name[:60]}: 旧串命中 {src.count(old)} 次，跳过', flush=True); continue
    shutil.copy(path, path + '.orig')
    try:
        open(path, 'w').write(src.replace(old, new, 1))
        env = dict(os.environ, CARGO_TARGET_DIR=W + '/target-' + os.environ.get('TREE','tree-m').split('-')[1])
        args = ['nice', '-n', '19', 'bash', R + '/run-with-memory-cap.sh', '6G', 'bash', R + '/capped.sh', '8', 'cargo', 'test', '--release', '--offline'] + cmd.split()
        r = subprocess.run(args, cwd=T, env=env, capture_output=True, text=True)
        out = r.stdout + r.stderr
        if 'error[' in out and 'could not compile' in out:
            print(f'行{ln} {name[:60]}: 编不过 {[l for l in out.splitlines() if l.startswith("error")][:1]}', flush=True); continue
        failed = re.findall(r'^test (\S+) \.\.\. FAILED', out, re.M)
        res = re.findall(r'^test result: .*', out, re.M)
        print(f'行{ln} {name[:70]}: rc={r.returncode} 红 {len(failed)} 条 {" ".join(x[-60:] for x in failed[:2])} | {res[-1][:70] if res else "无结果行"} ({time.time()-t0:.0f}s)', flush=True)
    finally:
        shutil.move(path + '.orig', path)
