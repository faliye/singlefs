#!/usr/bin/env python3
# 用法：python3 -I adhoc.py <仓副本根> <主仓根> <用例文件.tsv> [用例号…] —— 逐个用例把「文件 / 原文 / 替换文」打进副本、跑它列的每一组 cargo 参数、再还原。
# 用例文件每行五段（制表符）：用例号、文件、原文、替换文、cargo 参数组（多组用 ';;' 隔开）；原文 / 替换文里的 \n 表示换行；原文在文件里必须恰好命中一次。
# 输出每组一行：ADHOC <号> <组序> rc=<退出码> <结局> nfailed=<红的条数> failed=<红的测试名前 70 字，最多 3 个>。内存包装 6G（含 random_histories 的 8G），线程 15，一次一条 cargo。
import os, re, subprocess, sys
repo, scripts_repo, cases_file = sys.argv[1:4]
want = set(sys.argv[4:])
for line in open(cases_file, encoding='utf-8'):
    if not line.strip() or line.startswith('#'):
        continue
    case, path, old, new, arg_groups = line.rstrip('\n').split('\t')
    if want and case not in want:
        continue
    p = os.path.join(repo, path)
    orig = open(p, encoding='utf-8').read()
    o, n = old.replace('\\n', '\n'), new.replace('\\n', '\n')
    if orig.count(o) != 1:
        print(f'ADHOC {case} ANCHOR-HITS={orig.count(o)}', flush=True)
        continue
    try:
        open(p, 'w', encoding='utf-8').write(orig.replace(o, n))
        for group_index, args in enumerate(arg_groups.split(';;')):
            cap = '8G' if 'random_histories' in args else '6G'
            cmd = ['nice', '-n', '19', 'bash', scripts_repo + '/scripts/hosts/run-with-memory-cap.sh', cap,
                   'bash', scripts_repo + '/scripts/hosts/capped.sh', '15', 'cargo', 'test', '--offline'] + args.split()
            try:
                r = subprocess.run(cmd, cwd=repo, capture_output=True, text=True, timeout=235)
                out, rc = r.stdout + r.stderr, r.returncode
            except subprocess.TimeoutExpired:
                out, rc = '', 'TIMEOUT'
            failed = re.findall(r'^test (\S+) \.\.\. FAILED', out, re.M)
            verdict = ('TIMEOUT(不算结果)' if rc == 'TIMEOUT' else 'COMPILE-ERROR' if ('error[E' in out or 'could not compile' in out)
                       else 'GREEN(mutation survived)' if rc == 0 else 'RED')
            print(f'ADHOC {case} {group_index} rc={rc} {verdict} nfailed={len(failed)} failed={sorted(x[-70:] for x in failed)[:3]}', flush=True)
    finally:
        open(p, 'w', encoding='utf-8').write(orig)
