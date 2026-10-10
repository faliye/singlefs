#!/usr/bin/env python3
"""mutworker.py <树> <编译目录> <行文件.tsv> <结果文件> <预算秒> [内存上限=4G] [线程=5]
行文件六列（名、文件、原文、替换文、cargo test 参数、必须红的测试名；\\n 是换行）。逐行施加到 <树> 里，经内存包装跑 cargo test <参数>（debug，与 prove-red 同），
结果追加到 <结果文件>（一行：名 TAB 结局 TAB 红的测试名…）；已在结果文件里的名跳过；预算秒用完前不再开新的一行。"""
import os, re, shlex, subprocess, sys, time
tree, target, rows, done, budget = sys.argv[1:6]
mem = sys.argv[6] if len(sys.argv) > 6 else '4G'
threads = sys.argv[7] if len(sys.argv) > 7 else '5'
R = '/home/fy5090/code/singlefs/research/scripts'
start = time.time()
seen = set()
if os.path.exists(done):
    for line in open(done):
        seen.add(line.split('\t')[0])
def unesc(s): return s.replace('\\n', '\n')
for line in open(rows):
    cols = line.rstrip('\n').split('\t')
    if len(cols) < 6: continue
    name, f, old, new, args, test = cols[:6]
    if name in seen: continue
    if time.time() - start > float(budget): break
    path = os.path.join(tree, f)
    src = open(path).read()
    old_u, new_u = unesc(old), unesc(new)
    n = src.count(old_u)
    if n != 1:
        open(done, 'a').write(f'{name}\t旧串命中{n}次\n'); continue
    open(path, 'w').write(src.replace(old_u, new_u, 1))
    t0 = time.time()
    try:
        cmd = ['nice', '-n', '19', 'bash', f'{R}/run-with-memory-cap.sh', mem, 'bash', f'{R}/capped.sh', threads, 'cargo', 'test', '--offline'] + shlex.split(args)
        r = subprocess.run(cmd, cwd=tree, env=dict(os.environ, CARGO_TARGET_DIR=target), capture_output=True, text=True, timeout=600)
        out = r.stdout + r.stderr
        failed = re.findall(r'^test (\S+) \.\.\. FAILED', out, re.M)
        compile_err = bool(re.search(r'^error(\[E\d+\])?:', out, re.M)) and not re.search(r'^test result:', out, re.M)
        if compile_err: res = '编不过'
        elif test and any(x.endswith(test) for x in failed): res = '红(点名的)'
        elif failed: res = '红(别的)'
        elif r.returncode in (250, 252, 253, 254): res = f'包装退{r.returncode}'
        else: res = '绿'
        results = ' '.join(x[-80:] for x in failed[:6])
    except subprocess.TimeoutExpired:
        res, results = '超时', ''
    finally:
        open(path, 'w').write(src)
        os.utime(path)
    open(done, 'a').write(f'{name}\t{res}\t{results}\t{time.time()-t0:.0f}s\n')
