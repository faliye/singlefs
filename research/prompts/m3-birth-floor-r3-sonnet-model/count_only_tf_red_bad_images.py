#!/usr/bin/env python3
"""数第二轮攻方 logs/bad-r1six.log 里：池级 checker 0 红而 TF（CS2:tf-lenient）红的坏镜像有几张，按份名分。
用法：python3 count_only_tf_red_bad_images.py <bad-r1six.log>"""
import re, sys
from collections import Counter
pool, tf = {}, {}
for line in open(sys.argv[1], encoding="utf-8"):
    m = re.match(r"BI (BI-\S+) slot=(\d+) class=(\d+) key_width=(\d+) counts=\S+ \S+ \S+ \S+ pool_checker_violated=(\[.*\])", line)
    if m:
        pool[(m.group(1), m.group(2))] = m.group(5)
    m = re.match(r"BI (BI-\S+) slot=(\d+) class=(\d+) key_width=(\d+) counts=\S+ \S+ \S+ \S+ CS2:tf-lenient evaluated=\d+ violations=(\d+)", line)
    if m:
        tf[(m.group(1), m.group(2))] = int(m.group(5))
only = [k for k, p in pool.items() if p == "[]" and tf.get(k, 0) > 0]
print(f"坏镜像 {len(pool)} 张；池级 checker 红的 {sum(1 for p in pool.values() if p != '[]')} 张；池级 0 红而 TF 红的 {len(only)} 张")
print(dict(Counter(k[0] for k in only)))
