#!/usr/bin/env python3
"""第 29 批追加的 13 行变异（mutations.tsv 第 2977–2989 行）的静态核：锚点恰好命中一次、点名的测试在测试文件里有、参数筛选得到它。
用法：rowcheck.py <树根>   每行一条 ROWCHECK 输出（制表符分隔）：行号、名字前 30 字、锚点命中数、测试名在哪个文件、筛选结论。
筛选：带 --exact 的要与测试名全等；别的按 cargo test 的子串筛（-- 之后不以 - 开头的每个词，任一个是测试名的子串就选中）。"""
import re, sys, os
tree = sys.argv[1]
lines = open(os.path.join(tree, 'crates/mutations.tsv'), encoding='utf-8').read().split('\n')
files = {'harness': 'crates/singlefs-harness/tests/checker_known_bad_images.rs',
         'tier': 'crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs'}
src = {k: open(os.path.join(tree, v), encoding='utf-8').read() for k, v in files.items()}
for n in range(2977, 2990):
    c = lines[n - 1].split('\t')
    name, f, a, b, cmd, test = c[0], c[1], c[2].replace('\\n', '\n'), c[3].replace('\\n', '\n'), c[4], c[5]
    hits = open(os.path.join(tree, f), encoding='utf-8').read().count(a)
    which = 'tier' if '-p singlefs-checker-tier' in cmd else 'harness'
    present = len(re.findall(r'^fn %s\(' % re.escape(test), src[which], re.M)) + len(re.findall(r'^fn %s\(\n' % re.escape(test), src[which], re.M)) * 0
    toks = cmd.split()
    after = toks[toks.index('--') + 1:] if '--' in toks else []
    exact = '--exact' in after
    words = [w for w in after if not w.startswith('-')]
    if not words:
        sel = 'all'
    elif exact:
        sel = 'selected' if test in words else 'NOT selected'
    else:
        sel = 'selected' if any(w in test for w in words) else 'NOT selected'
    print('ROWCHECK\t%d\t%s\tanchor_hits=%d\ttest_fn_in_%s=%d\t%s' % (n, name[:30], hits, which, present, sel))
