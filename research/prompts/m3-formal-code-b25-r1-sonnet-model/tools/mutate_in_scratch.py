#!/usr/bin/env python3
"""在草稿副本里施加一条变异、跑点名的 harness 档测试、还原。

用法：mutate_in_scratch.py <副本根> <变异表> <行名前缀|--custom 文件 原文 替换文 cargo参数 测试名>
变异表列：名字 / 文件 / 原文 / 替换文 / cargo 参数 / 测试名（`\\n` 还原成换行，别的反斜杠按字面，同 mutate.sh）。
锚点在文件里必须恰好命中 1 次，否则退 3（不跑）。
测试经 capped.sh 与 run-with-memory-cap.sh 跑。输出一行 RESULT。
"""
import os, subprocess, sys, shutil

def unescape(s):
    return s.replace('\\n', '\n')

def run_one(root, name, rel, old, new, cargo_args, test):
    path = os.path.join(root, rel)
    text = open(path, encoding='utf-8').read()
    n = text.count(old)
    if n != 1:
        print(f"RESULT name={name} anchor_hits={n} outcome=not_run")
        return 3
    backup = path + '.orig-for-mutation'
    shutil.copy2(path, backup)
    try:
        with open(path, 'w', encoding='utf-8') as f:
            f.write(text.replace(old, new))
        env = dict(os.environ)
        env['RUN_WITH_MEMORY_CAP_PARALLEL_REASON'] = '派出去的正推腿与攻方腿各跑各的，同会话号'
        env['RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON'] = '草稿副本里施加变异再跑，上限取派发默认的 8G，峰值表是别处测的'
        repo = os.environ.get('SINGLEFS_REPO', '/home/fy5090/code/singlefs')
        cmd = ['nice', '-n', '19', 'bash', f'{repo}/scripts/hosts/capped.sh', '6', 'bash',
               f'{repo}/scripts/hosts/run-with-memory-cap.sh', os.environ.get('MEMORY_CAP', '8G'),
               'cargo', 'test', '--offline'] + cargo_args.split() + ['--', test]
        p = subprocess.run(cmd, cwd=root, env=env, capture_output=True, text=True, timeout=230)
        out = p.stdout + p.stderr
        logdir = os.environ.get('LOG_DIR')
        if logdir:
            os.makedirs(logdir, exist_ok=True)
            safe = ''.join(c if c.isalnum() else '_' for c in name)[:40]
            open(os.path.join(logdir, safe + '-' + str(abs(hash(name)) % 10000) + '.log'), 'w', encoding='utf-8').write(out)
        failed = [l.strip() for l in out.splitlines() if l.startswith('test ') and l.rstrip().endswith('FAILED')]
        passed = [l.strip() for l in out.splitlines() if l.startswith('test ') and l.rstrip().endswith('ok')]
        compile_err = 'could not compile' in out
        outcome = 'invalid_compile_error' if compile_err else ('caught' if failed else ('not_caught' if passed else 'unknown'))
        print(f"RESULT name={name} anchor_hits=1 exit={p.returncode} outcome={outcome} failed={len(failed)} passed={len(passed)}")
        if outcome == 'unknown':
            print('  RAW_TAIL', out[-600:].replace('\n', ' | '))
        for l in failed:
            print("  FAILED_LINE", l)
        return 0
    finally:
        shutil.move(backup, path)

def main():
    root = sys.argv[1]
    if sys.argv[2] == '--custom':
        _, _, _, rel, old, new, cargo_args, test = sys.argv[:8] + [None] * 0
        name = 'custom:' + rel
        return run_one(root, name, rel, unescape(old), unescape(new), cargo_args, test)
    table, prefix = sys.argv[2], sys.argv[3]
    rc = 0
    for line in open(table, encoding='utf-8'):
        cols = line.rstrip('\n').split('\t')
        if len(cols) >= 6 and cols[0].startswith(prefix):
            rc |= run_one(root, cols[0][:60], cols[1], unescape(cols[2]), unescape(cols[3]), cols[4], cols[5])
    return rc

sys.exit(main())
