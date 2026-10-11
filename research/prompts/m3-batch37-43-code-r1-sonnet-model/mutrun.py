#!/usr/bin/env python3
# 用法：python3 -I mutrun.py <仓副本根> <主仓根（取 scripts/）> <rows.txt>   —— 逐行把变异表里名字以前缀（rows.txt 每行一个）起头的行打进副本、跑它的 cargo test、再还原，每行出一行 ROW。
# 环境：CARGO_TARGET_DIR 由调用方设；ROWS_FROM / ROWS_TO 取序号区间（序号 = 命中行在变异表里的先后）；checker 档行（-p singlefs-checker-tier）不跑，打 SKIPPED；
#       内存包装 6G（编译加跑），random_histories 的行 8G；线程 15（capped.sh）；一次只起一条 cargo。
import os, re, subprocess, sys
repo, scripts_repo, rows_file = sys.argv[1], sys.argv[2], sys.argv[3]
prefixes = [l.rstrip('\n') for l in open(rows_file, encoding='utf-8') if l.strip()]
table = os.path.join(repo, 'crates/mutations.tsv')
rows = [l.rstrip('\n').split('\t') for l in open(table, encoding='utf-8') if l.count('\t') == 5 and any(l.startswith(p) for p in prefixes)]
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
    cap = '8G' if 'random_histories' in args else '6G'
    try:
        open(p, 'w', encoding='utf-8').write(orig.replace(o, n))
        cmd = ['nice', '-n', '19', 'bash', scripts_repo + '/scripts/run-with-memory-cap.sh', cap,
               'bash', scripts_repo + '/scripts/capped.sh', '15', 'cargo', 'test', '--offline'] + args.split()
        r = subprocess.run(cmd, cwd=repo, capture_output=True, text=True, timeout=235)
        out = r.stdout + r.stderr
        rc = r.returncode
    except subprocess.TimeoutExpired:
        out, rc = '', 'TIMEOUT'
    finally:
        open(p, 'w', encoding='utf-8').write(orig)
    failed = re.findall(r'^test (\S+) \.\.\. FAILED', out, re.M)
    if rc == 'TIMEOUT':
        verdict = 'TIMEOUT(不算结果)'
    elif rc in (245,) or 250 <= rc <= 254:
        verdict = f'WRAPPER-OUTCOME(不算结果)'
    elif 'error[E' in out or 'could not compile' in out:
        verdict = 'COMPILE-ERROR'
    elif rc == 0:
        verdict = 'GREEN(mutation survived)'
    elif all(any(e == x or x.endswith('::' + e) for x in failed) for e in tests.split()):
        verdict = 'RED(expected test failed)'
    else:
        verdict = 'RED(but expected test not among failures)'
    print(f'ROW {idx} rc={rc} {verdict} nfailed={len(failed)} name={name[:60]}', flush=True)
