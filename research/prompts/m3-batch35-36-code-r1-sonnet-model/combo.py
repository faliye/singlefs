#!/usr/bin/env python3
# 用法：python3 -I combo.py <仓副本根> <主仓根> <行名前缀甲> <行名前缀乙> —— 同时打进两行变异（各自在变异表里按名字前缀恰好命中一行），跑甲行点名的 cargo test，再还原。
# 用来看一行变异「存活」是不是被另一处改法盖住：甲行单独存活，甲乙一起应当红。内存包装 8G、线程 7。
import os, re, subprocess, sys
repo, scripts_repo, pa, pb = sys.argv[1:5]
rows = [l.rstrip('\n').split('\t') for l in open(os.path.join(repo, 'crates/mutations.tsv'), encoding='utf-8')]
def pick(prefix):
    hits = [r for r in rows if r[0].startswith(prefix) and len(r) == 6]
    assert len(hits) == 1, (prefix, len(hits))
    return hits[0]
a, b = pick(pa), pick(pb)
saved = {}
try:
    for name, path, old, new, args, tests in (a, b):
        p = os.path.join(repo, path)
        text = saved.setdefault(p, open(p, encoding='utf-8').read())
        cur = open(p, encoding='utf-8').read()
        o, n = old.replace('\\n', '\n'), new.replace('\\n', '\n')
        assert cur.count(o) == 1, ('anchor', name[:40])
        open(p, 'w', encoding='utf-8').write(cur.replace(o, n))
    cmd = ['nice', '-n', '19', 'bash', scripts_repo + '/research/scripts/run-with-memory-cap.sh', '8G', 'bash', scripts_repo + '/research/scripts/capped.sh', '7', 'cargo', 'test', '--offline'] + a[4].split()
    r = subprocess.run(cmd, cwd=repo, capture_output=True, text=True, timeout=220)
    out = r.stdout + r.stderr
finally:
    for p, text in saved.items():
        open(p, 'w', encoding='utf-8').write(text)
failed = re.findall(r'^test (\S+) \.\.\. FAILED', out, re.M)
print(f'COMBO rc={r.returncode} failed={sorted(x[:80] for x in failed)[:3]} a={a[0][:40]} b={b[0][:40]}')
