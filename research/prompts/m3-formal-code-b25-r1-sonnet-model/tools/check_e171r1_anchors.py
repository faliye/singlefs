#!/usr/bin/env python3
"""核 crates/mutations.tsv 里「E171R1 装置」开头各行：锚点在装置里恰好命中 1 次、替换文与原文不同、点名的测试在装置里是 #[test] 函数。
用法：check_e171r1_anchors.py <树根>
变异表规则同 mutate.sh：`\\n` 还原成换行，别的反斜杠按字面。不跑任何测试。
"""
import re, sys
root = sys.argv[1]
rows = [l.rstrip('\n').split('\t') for l in open(f'{root}/crates/mutations.tsv', encoding='utf-8') if l.startswith('E171R1 装置')]
bad = 0
cache = {}
for cols in rows:
    name, file, old, new, args, test = cols[:6]
    text = cache.setdefault(file, open(f'{root}/{file}', encoding='utf-8').read())
    o, n = old.replace('\\n', '\n'), new.replace('\\n', '\n')
    hits = text.count(o)
    test_is_fn = re.search(r'#\[test\]\s*(?:#\[[^\]]*\]\s*)*fn ' + re.escape(test) + r'\b', text) is not None
    ok = hits == 1 and o != n and test_is_fn
    bad += 0 if ok else 1
    print(f'{"ok " if ok else "BAD"} anchor_hits={hits} differs={o != n} test_is_fn={test_is_fn} {name[:50]} -> {test[:60]}')
print(f'rows={len(rows)} bad={bad}')
sys.exit(1 if bad else 0)
