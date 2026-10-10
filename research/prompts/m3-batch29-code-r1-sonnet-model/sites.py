#!/usr/bin/env python3
"""判不了原因字的站点号：把 crates/singlefs-checker-tier/src 下各 .wgsl 里「(N << 8u)」的字面站点号与 d_push_container_tree_entries_in_reverse 的实参站点号并起来数，
报重复的与 37、38（第 29 批新加）的出现次数。用法：sites.py <树根>"""
import re, sys, os, glob, collections
d = os.path.join(sys.argv[1], 'crates/singlefs-checker-tier/src')
count = collections.Counter()
where = collections.defaultdict(list)
for p in sorted(glob.glob(d + '/*.wgsl')):
    t = open(p, encoding='utf-8').read()
    for m in re.finditer(r'\((\d+)u << 8u\)', t):
        count[int(m.group(1))] += 1; where[int(m.group(1))].append(os.path.basename(p))
    for m in re.finditer(r'd_push_container_tree_entries_in_reverse\([^,]+, [^,]+, (\d+)u\)', t):
        count[int(m.group(1))] += 1; where[int(m.group(1))].append(os.path.basename(p) + ':arg')
dups = sorted(k for k, v in count.items() if v > 1)
print('SITES\tdistinct=%d\tmax=%d\tduplicates=%s' % (len(count), max(count), dups))
for k in (37, 38, 61):
    print('SITES\tsite %d\toccurrences=%d\t%s' % (k, count[k], where[k]))
