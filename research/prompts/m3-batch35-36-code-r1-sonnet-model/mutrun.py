#!/usr/bin/env python3
# 用法：python3 -I mutrun.py <仓副本根> <主仓根（取 research/scripts/）> <名字前缀，竖线分隔多个> —— 逐行把变异表里名字以前缀起头的行打进副本、跑它的 cargo test、再还原，每行出一行 ROW。
# 环境：CARGO_TARGET_DIR 由调用方设；ROWS_FROM / ROWS_TO 取行区间；内存包装 8G、线程 7（capped.sh）；一次只起一条 cargo；checker 档行（-p singlefs-checker-tier）不跑。
import os, re, subprocess, sys
repo, scripts_repo, prefixes = sys.argv[1], sys.argv[2], sys.argv[3].split('|')
table = os.path.join(repo, 'crates/mutations.tsv')
rows = [l.rstrip('\n').split('\t') for l in open(table, encoding='utf-8') if any(l.startswith(p) for p in prefixes)]
print('ROWS', len(rows), flush=True)
lo, hi = int(os.environ.get('ROWS_FROM', 0)), int(os.environ.get('ROWS_TO', 10**9))
for idx, (name, path, old, new, args, tests) in enumerate(rows):
    if not lo <= idx < hi:
        continue
    if 'singlefs-checker-tier' in args:
        print(f'ROW {idx} SKIPPED（checker 档，子 agent 不跑） name={name[:60]}', flush=True)
        continue
    p = os.path.join(repo, path)
    orig = open(p, encoding='utf-8').read()
    o, n = old.replace('\\n', '\n'), new.replace('\\n', '\n')
    hits = orig.count(o)
    if hits != 1:
        print(f'ROW {idx} ANCHOR-HITS={hits} name={name[:60]}', flush=True)
        continue
    try:
        open(p, 'w', encoding='utf-8').write(orig.replace(o, n))
        cmd = ['nice', '-n', '19', 'bash', scripts_repo + '/research/scripts/run-with-memory-cap.sh', '8G',
               'bash', scripts_repo + '/research/scripts/capped.sh', '7', 'cargo', 'test', '--offline'] + args.split()
        r = subprocess.run(cmd, cwd=repo, capture_output=True, text=True, timeout=220)
        out = r.stdout + r.stderr
    finally:
        open(p, 'w', encoding='utf-8').write(orig)
    failed = re.findall(r'^test (\S+) \.\.\. FAILED', out, re.M)
    if 'error[E' in out or 'could not compile' in out:
        verdict = 'COMPILE-ERROR'
    elif r.returncode == 0:
        verdict = 'GREEN(mutation survived)'
    elif all(any(e == x or x.endswith('::' + e) for x in failed) for e in tests.split()):
        verdict = 'RED(expected test failed)'
    else:
        verdict = 'RED(but expected test not among failures)'
    print(f'ROW {idx} rc={r.returncode} {verdict} failed={sorted(x[:80] for x in failed)[:3]} name={name[:60]}', flush=True)
