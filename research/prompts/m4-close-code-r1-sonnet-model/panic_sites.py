#!/usr/bin/env python3
"""正推腿 T6/T7 自带比对：逐文件抽出非测试代码里能 panic 的写法（assert!/assert_eq!/assert_ne!/.expect(/unreachable!/panic!/unwrap()），
连同所在 fn，改前（git show <提交>:路径）改后（冻结副本）按多重集比。
用法：panic_sites.py <冻结副本根> <改前取自的提交> <仓根> 文件名… （文件名不带 .rs，相对 crates/singlefs-core/src/）"""
import re, sys, collections, subprocess
frozen, commit, repo = sys.argv[1:4]
pat = re.compile(r'assert!|assert_eq!|assert_ne!|\.expect\(|unreachable!|panic!|unwrap\(\)')
def sites(text):
    out = []; fn = '?'; intest = False
    for i, line in enumerate(text.split('\n'), 1):
        if re.match(r'\s*#\[cfg\(test\)\]', line): intest = True
        m = re.search(r'\bfn\s+(\w+)', line)
        if m and not line.strip().startswith('//'): fn = m.group(1)
        if line.strip().startswith('//'): continue
        if pat.search(line) and not intest:
            out.append((fn, re.sub(r'\s+', ' ', line.strip()), i))
    return out
for f in sys.argv[4:]:
    rel = 'crates/singlefs-core/src/%s.rs' % f
    head = subprocess.run(['git', '-C', repo, 'show', '%s:%s' % (commit, rel)], capture_output=True, text=True).stdout
    new = open('%s/%s' % (frozen, rel), encoding='utf-8').read()
    h, n = sites(head), sites(new)
    ch = collections.Counter((a, b) for a, b, _ in h); cn = collections.Counter((a, b) for a, b, _ in n)
    print('==', f, 'HEAD', len(h), 'NEW', len(n))
    for k, v in (cn - ch).items(): print('  + ', k[0], '|', k[1][:150], 'x%d' % v, 'line', [l for a, b, l in n if (a, b) == k][:3])
    for k, v in (ch - cn).items(): print('  - ', k[0], '|', k[1][:150], 'x%d' % v)
