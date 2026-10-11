#!/usr/bin/env python3
# 用法：python3 -I adhoc.py <仓副本根> <主仓根> <文件> <原文（\n 表换行）> <替换文> <cargo test 的参数…> —— 临时打一处变异、跑那几条 cargo test、还原；不在变异表里的探针用它。内存包装 8G、线程 7。
import os, re, subprocess, sys
repo, scripts_repo, path, old, new = sys.argv[1:6]
args = sys.argv[6:]
p = os.path.join(repo, path)
orig = open(p, encoding='utf-8').read()
o, n = old.replace('\\n', '\n'), new.replace('\\n', '\n')
hits = orig.count(o)
if hits != 1:
    print(f'ADHOC ANCHOR-HITS={hits}'); sys.exit(2)
try:
    open(p, 'w', encoding='utf-8').write(orig.replace(o, n))
    cmd = ['nice', '-n', '19', 'bash', scripts_repo + '/scripts/run-with-memory-cap.sh', '8G', 'bash', scripts_repo + '/scripts/capped.sh', '7', 'cargo', 'test', '--offline'] + args
    r = subprocess.run(cmd, cwd=repo, capture_output=True, text=True, timeout=230)
    out = r.stdout + r.stderr
finally:
    open(p, 'w', encoding='utf-8').write(orig)
failed = re.findall(r'^test (\S+) \.\.\. FAILED', out, re.M)
compile_error = 'error[E' in out or 'could not compile' in out
print(f'ADHOC rc={r.returncode} compile_error={compile_error} failed={sorted(x[:90] for x in failed)[:6]} nfailed={len(failed)}')
