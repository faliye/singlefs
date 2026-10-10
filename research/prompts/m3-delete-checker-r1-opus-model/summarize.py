#!/usr/bin/env python3
"""汇总 m3dc 原型的输出：合法历史上每个候选红几次（按最新根 / 别的根分开），改坏的镜像上每个候选红在哪。"""
import collections, glob, os, re, sys

out = sys.argv[1]
def counts(paths):
    table = collections.defaultdict(lambda: [0, 0, 0])
    for path in paths:
        for line in open(path, encoding='utf-8'):
            if line.startswith('M3DC_COUNT '):
                name, place, h, r, n = line[len('M3DC_COUNT '):].rstrip('\n').split('\t')
                kind = place.split('-', 1)[1]
                row = table[(name, kind)]
                row[0] += int(h.split('=')[1]); row[1] += int(r.split('=')[1]); row[2] += int(n.split('=')[1])
    return table

def summary_lines(paths):
    return [re.sub(r' elapsed_seconds=[0-9.]+', '', l.rstrip('\n')) for p in paths for l in open(p, encoding='utf-8') if l.startswith('M3DC_SUMMARY')]

legal = sorted(glob.glob(os.path.join(out, 'legal2-b*.log'))) + sorted(glob.glob(os.path.join(out, 'random-*.log')))
print('## 合法历史')
for l in summary_lines(legal):
    print(l)
endings = collections.Counter()
for p in legal:
    for l in open(p, encoding='utf-8'):
        if ' ENDING ' in l:
            endings[l.split(' ENDING ', 1)[1].strip()[:40]] += 1
print('endings', dict(endings))
checker = collections.Counter()
for p in legal:
    for l in open(p, encoding='utf-8'):
        if l.startswith('M3DC_CHECKER_RED'):
            name, c = l.split()[1:3]; checker[name] += int(c)
print('existing checker reds on legal images:', dict(checker))
t = counts(legal)
names = sorted({k[0] for k in t})
print('| 候选 | 最新根 成立/红/不适用 | 别的根（≥F）成立/红/不适用 | 别的根（<F，不在候选集）成立/红/不适用 |')
print('|---|---|---|---|')
for n in names:
    a = t[(n, 'newest')]; b = t[(n, 'older>=F')]; c = t[(n, 'older<F')]
    print(f'| {n} | {a[0]}/{a[1]}/{a[2]} | {b[0]}/{b[1]}/{b[2]} | {c[0]}/{c[1]}/{c[2]} |')

print('\n## 改坏的镜像（每种改坏 × 每个批大小 × 每种内容一段历史，unlink 那一步之后的镜像，判环里每条根）')
muts = sorted(glob.glob(os.path.join(out, 'mut-*-b*.log')))
by_mut = collections.defaultdict(list)
for p in muts:
    m = re.match(r'mut-(.+)-b(\d+)\.log', os.path.basename(p))
    by_mut[m.group(1)].append(p)
for mut, paths in sorted(by_mut.items()):
    t = counts(paths)
    checker = collections.Counter()
    for p in paths:
        for l in open(p, encoding='utf-8'):
            if l.startswith('M3DC_CHECKER_RED'):
                name, c = l.split()[1:3]; checker[name] += int(c)
    hists = sum(int(re.search(r'histories=(\d+)', l).group(1)) for l in summary_lines(paths))
    older = lambda n: t[(n, 'older>=F')][1] + t[(n, 'older<F')][1]
    reds = [(n, t[(n, 'newest')][1], older(n)) for n in sorted({k[0] for k in t}) if t[(n, 'newest')][1] or older(n)]
    print(f'### {mut}（{len(paths)} 份输出、{hists} 段历史）')
    print('existing checker reds:', dict(checker))
    for n, a, b in reds:
        print(f'  {n}: 最新根红 {a} 次、别的根红 {b} 次')
