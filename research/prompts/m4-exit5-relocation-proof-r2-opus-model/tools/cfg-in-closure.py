#!/usr/bin/env python3
"""发布、回退两条闭包里 core 的 fn，函数体（按闭包给的起止行）里出现几种 cfg 写法：Verus 只按一组 cfg 验，产品与测试构建各取各的。"""
import re, sys
work, tree = sys.argv[1], sys.argv[2]
rows = {}
for name in ("publish", "roll_back_by_a_forward_publish"):
    for l in open(f"{work}/closure-{name}.txt"):
        kind, full, place, _ = l.rstrip("\n").split("\t")
        if kind == "fn" and full.startswith("singlefs_core::"):
            rows[full] = place
cache = {}
counts = {}
pattern = re.compile(r'cfg!?\s*\(\s*(feature\s*=\s*"[^"]+"|test|debug_assertions|not\s*\(\s*feature\s*=\s*"[^"]+"\s*\))')
for full, place in sorted(rows.items()):
    path, span = place.rsplit(":", 1)
    a, b = map(int, span.split("-"))
    lines = cache.setdefault(path, open(f"{tree}/{path}").read().split("\n"))
    body = "\n".join(lines[a - 1:b])
    for m in set(pattern.findall(body)):
        key = re.sub(r"\s+", "", m)
        counts.setdefault(key, []).append(full.rsplit("::", 1)[1])
for key, fns in sorted(counts.items()):
    print(f"name=cfg-in-closure-fn-bodies cfg={key} fns={len(fns)} first={sorted(fns)[:4]}")
print(f"name=cfg-in-closure-fn-bodies total_core_fns={len(rows)}")
