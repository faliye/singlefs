#!/usr/bin/env python3
# 用法：python3 -I mutrun.py <仓副本根> <包装脚本所在仓根> <变异表取行的名字前缀> —— 逐行把变异表里名字以前缀起头的行打进副本、跑它的 cargo test、再还原，每行出一行 ROW。
# 例：python3 -I mutrun.py $WORK/repo $SINGLEFS_REPO 第33批T   （前缀写成 @文件 时改读那份 tsv 的每一行，例 @replaced20.tsv：diff 里替换的 20 行，其中 4 行在 checker 档、不要跑）
# 环境：CARGO_TARGET_DIR 由调用方设；内存包装 4G、线程 10（capped.sh）；一次只起一条 cargo。
import os, re, subprocess, sys
repo, scripts_repo, prefix = sys.argv[1], sys.argv[2], sys.argv[3]
table = prefix[1:] if prefix.startswith('@') else os.path.join(repo, 'crates/mutations.tsv')
rows = [l.rstrip('\n').split('\t') for l in open(table, encoding='utf-8') if (prefix.startswith('@') and l.strip()) or l.startswith(prefix)]
print('ROWS', len(rows), flush=True)
lo, hi = int(os.environ.get('ROWS_FROM', 0)), int(os.environ.get('ROWS_TO', 10**9))
for idx, (name, path, old, new, args, tests) in enumerate(rows):
    if not lo <= idx < hi:
        continue
    if 'singlefs-checker-tier' in args:
        print(f'ROW {idx} SKIPPED（checker 档，子 agent 不跑） name={name[:70]}', flush=True)
        continue
    p = os.path.join(repo, path)
    orig = open(p, encoding='utf-8').read()
    o, n = old.replace('\\n', '\n'), new.replace('\\n', '\n')
    assert orig.count(o) == 1, ('anchor', idx)
    try:
        open(p, 'w', encoding='utf-8').write(orig.replace(o, n))
        cmd = ['nice', '-n', '19', 'bash', scripts_repo + '/scripts/run-with-memory-cap.sh', '4G',
               'bash', scripts_repo + '/scripts/capped.sh', '10', 'cargo', 'test'] + args.split()
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
    print(f'ROW {idx} rc={r.returncode} {verdict} failed={sorted(x[:90] for x in failed)[:4]} name={name[:70]}', flush=True)
