#!/usr/bin/env python3
"""在冻结副本的拷贝里逐行施加 mutations.tsv 的变异（原文必须恰好命中一次），跑该行的测试，红（退出码非 0）才算会红。
用法：mutate_rows.py <拷贝根> <行号> ... ；每行施加后还原。stdout 一行一结果：行号 TAB 命中数 TAB 退出码 TAB 红了的测试数。"""
import subprocess, sys, os, re
root = sys.argv[1]
rows = [int(x) for x in sys.argv[2:]]
lines = open(os.environ.get('MUTATIONS_TSV') or os.path.join(root, 'crates/mutations.tsv'), encoding='utf-8').read().split('\n')
repo = os.environ['SINGLEFS_REPO']
target = os.environ['CARGO_TARGET_DIR']
for n in rows:
    cols = lines[n - 1].split('\t')
    name, rel, orig, mut, args = cols[0], cols[1], cols[2].replace('\\n', '\n'), cols[3].replace('\\n', '\n'), cols[4]
    path = os.path.join(root, rel)
    src = open(path, encoding='utf-8').read()
    hits = src.count(orig)
    if hits != 1:
        print(f"{n}\t{hits}\tANCHOR\t-"); continue
    open(path, 'w', encoding='utf-8').write(src.replace(orig, mut))
    try:
        cmd = ['bash', f'{repo}/scripts/run-with-memory-cap.sh', '4G', 'bash', f'{repo}/scripts/capped.sh', '10',
               'cargo', 'test', '--offline', '--manifest-path', f'{root}/Cargo.toml'] + args.split(' ')
        p = subprocess.run(cmd, capture_output=True, text=True)
        out = p.stdout + p.stderr
        failed = len(re.findall(r'^test .* \.\.\. FAILED', out, re.M))
        print(f"{n}\t{hits}\t{p.returncode}\t{failed}")
        open(os.path.join(os.environ.get('LOGDIR', '/tmp'), f'mut-{n}.log'), 'w').write(out)
    finally:
        open(path, 'w', encoding='utf-8').write(src)
