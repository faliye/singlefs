#!/usr/bin/env python3
"""正推腿 T10 自带比对，只读：（一）变异表 crates/mutations.tsv 每一行的「原文」在冻结副本里是不是恰好命中一次（锚点失效的行）；
（二）每个主题的变异行点名的测试 fn 在仓里是不是找得到；（三）T6/T7 文件里每个非测试 exec fn 有没有变异行的原文落在它的函数体里。
用法：mutation_checks.py <冻结副本根> <crates-sha256.txt 路径> <文件名.rs>…（第三项的文件，相对 crates/singlefs-core/src/）"""
import re, sys, os, collections
ROOT = sys.argv[1].rstrip('/') + '/'
scope = {l.split()[1] for l in open(sys.argv[2], encoding='utf-8') if l.strip()}
rows = []
for n, l in enumerate(open(ROOT + 'crates/mutations.tsv', encoding='utf-8'), 1):
    if l.startswith('#') or not l.strip(): continue
    p = l.rstrip('\n').split('\t')
    if len(p) >= 6: rows.append((n, p))
cache = {}
def text(f):
    if f not in cache: cache[f] = open(ROOT + f, encoding='utf-8', errors='replace').read()
    return cache[f]
bad = []
for n, p in rows:
    if not os.path.exists(ROOT + p[1]): bad.append((n, p[1], 'nofile', p[0][:50])); continue
    c = text(p[1]).count(p[2].replace('\\n', '\n'))
    if c != 1: bad.append((n, p[1], c, p[0][:50]))
print('（一）变异表共', len(rows), '行；原文在冻结副本里不是恰好命中一次的', len(bad), '行')
for b in bad: print('  锚点失效', b)
inscope = [r for r in rows if r[1][1] in scope]
print('    其中点名被判 30 份文件的行', len(inscope), '，失效', sum(1 for b in bad if b[1] in scope))
fns = set()
for dp, dn, fs in os.walk(ROOT + 'crates'):
    if '/target' in dp: continue
    for f in fs:
        if f.endswith('.rs'):
            fns.update(re.findall(r'\bfn\s+(\w+)', open(os.path.join(dp, f), encoding='utf-8', errors='replace').read()))
themes = {'多 inode 改 size': r'多 inode 改 size', '部分写': r'部分写', '发布触发': r'里程碑四发布触发', '块设备': r'discard|写零|打洞|块设备'}
print('（二）')
for t, rx in themes.items():
    sel = [(n, p) for n, p in rows if re.search(rx, p[0])]
    miss = [n for n, p in sel if p[5].strip() not in ('-', '') and p[5].split()[0].split('::')[-1] not in fns]
    print('  ', t, len(sel), '行；点名测试在仓里找不到 fn 的', len(miss), '行')
hdr = re.compile(r'^(\s*)(pub(\([a-z]+\))?\s+)?(const\s+)?(unsafe\s+)?fn\s+(\w+)')
print('（三）')
for f in sys.argv[3:]:
    path = 'crates/singlefs-core/src/' + f
    t = text(path); lines = t.split('\n'); cut = len(lines)
    for i, l in enumerate(lines):
        if re.match(r'\s*#\[cfg\(test\)\]', l): cut = i; break
    starts = [(i + 1, hdr.match(l).group(6)) for i, l in enumerate(lines[:cut]) if not re.search(r'\b(spec|proof)\s+fn\b', l) and hdr.match(l)]
    spans = [(starts[k][0], (starts[k + 1][0] - 1 if k + 1 < len(starts) else cut), starts[k][1]) for k in range(len(starts))]
    covered = set()
    for n, p in rows:
        if p[1] != path: continue
        pos = t.find(p[2].replace('\\n', '\n'))
        if pos < 0: continue
        line = t[:pos].count('\n') + 1
        covered.update(name for a, b, name in spans if a <= line <= b)
    unc = [name for a, b, name in spans if name not in covered]
    print('  %s：非测试 exec fn %d 个，有变异行落在函数体里的 %d 个；没有的：%s' % (f, len(spans), len(spans) - len(unc), ', '.join(unc[:40])))
