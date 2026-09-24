#!/usr/bin/env python3
"""T5 模型：一条「阶段里自带改动范围取法」的判据（候选乙 / 丙 都得写一条这样的判据），在真仓 gate.d 上命中哪些行。
判据 R：阶段文件的非注释行里（shell 与内嵌 python 都算；python 列表写法先把引号、逗号抹平再比），出现
  a) diff … --name-only   b) ls-files … --others   c) merge-base   d) @{upstream}   e) gate-ok
之一，且那一行里有 git 字样（给 --no-git-word 就不要求）。只读。用法：python3 t5-scan.py <仓根> [--no-git-word]
"""
import glob, os, re, sys
root = sys.argv[1]
patterns = {
    'a': re.compile(r'\bdiff\b.*--name-only'),
    'b': re.compile(r'\bls-files\b.*--others'),
    'c': re.compile(r'merge-base'),
    'd': re.compile(r'@\{upstream\}'),
    'e': re.compile(r'gate-ok'),
}
hits = []
for path in sorted(glob.glob(os.path.join(root, '.claude/gate.d/[0-9][0-9]-*.sh'))):
    for number, line in enumerate(open(path, encoding='utf-8'), 1):
        stripped = line.strip()
        if stripped.startswith('#') or not stripped:
            continue
        flat = re.sub(r'["\',\[\]]', ' ', stripped)
        keys = [k for k, p in patterns.items() if p.search(flat)]
        if keys and ('git' in flat or '--no-git-word' in sys.argv[2:]):
            hits.append((os.path.basename(path), number, ''.join(keys), stripped[:150]))
for h in hits:
    print(f'{h[0]}:{h[1]} [{h[2]}] {h[3]}')
print(f'命中 {len(hits)} 行，{len({h[0] for h in hits})} 个阶段')
