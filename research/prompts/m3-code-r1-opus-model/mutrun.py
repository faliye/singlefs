#!/usr/bin/env python3
"""m3-code-r1 攻方腿：在仓副本上逐条套一个改坏（旧串恰好命中一次），跑点名的 harness / core lib 测试，记哪些红，再还原。
用法：mutrun.py <副本根> <spec.tsv> <日志目录> [只跑的名字…]
spec 每行六列（制表符分隔）：名字、文件（相对副本根）、旧串、新串（\\n 表示换行）、cargo test 参数（空格分隔）、备注。
"""
import os, subprocess, sys, re, pathlib

root, spec, logdir = sys.argv[1], sys.argv[2], sys.argv[3]
only = set(sys.argv[4:])
pathlib.Path(logdir).mkdir(parents=True, exist_ok=True)
rows = []
for line in open(spec, encoding='utf-8'):
    if not line.strip() or line.startswith('#'):
        continue
    name, rel, old, new, args, note = line.rstrip('\n').split('\t')
    rows.append((name, rel, old.replace('\\n', '\n'), new.replace('\\n', '\n'), args.split(), note))
for name, rel, old, new, args, note in rows:
    if only and name not in only:
        continue
    path = os.path.join(root, rel)
    original = open(path, encoding='utf-8').read()
    hits = original.count(old)
    if hits != 1:
        print(f"MUT {name}\tSKIP\t旧串命中 {hits} 次", flush=True)
        continue
    try:
        open(path, 'w', encoding='utf-8').write(original.replace(old, new, 1))
        log = os.path.join(logdir, f"{name}.log")
        with open(log, 'w', encoding='utf-8') as out:
            rc = subprocess.run(['cargo', 'test', '--offline'] + args, cwd=root, stdout=out, stderr=subprocess.STDOUT).returncode
        text = open(log, encoding='utf-8', errors='replace').read()
        failed = sorted(set(re.findall(r'^    (\S+)$', text, re.M)))
        results = re.findall(r'^test result: .*$', text, re.M)
        compile_error = 'error[E' in text or 'could not compile' in text
        verdict = 'COMPILE_ERROR' if compile_error else ('RED' if rc != 0 else 'GREEN')
        print(f"MUT {name}\t{verdict}\trc={rc}\tfailed={failed}\tresults={len(results)}", flush=True)
    finally:
        open(path, 'w', encoding='utf-8').write(original)
