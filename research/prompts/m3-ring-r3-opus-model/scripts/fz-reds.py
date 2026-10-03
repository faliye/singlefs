#!/usr/bin/env python3
# 比两臂同种子的 checker 红：python3 fz-reds.py <today 文件…> -- <window 文件…>
import re, sys
args = sys.argv[1:]; i = args.index('--')
def load(paths):
    d = {}
    for p in paths:
        for l in open(p):
            if not l.startswith('FUZZ '): continue
            seed = int(re.search(r'seed=(\d+)', l).group(1))
            reds = re.search(r'checker_reds=(\[.*?\]) refusals', l).group(1)
            invs = sorted(set(re.findall(r'I-[0-9.]+', reds)))
            d[seed] = invs
    return d
t = load(args[:i]); w = load(args[i+1:])
from collections import Counter
ct = Counter(x for v in t.values() for x in v); cw = Counter(x for v in w.values() for x in v)
print('today 每条不变量红的段数', dict(ct)); print('window 每条不变量红的段数', dict(cw))
only_w = {s: w[s] for s in w if w[s] and set(w[s]) - set(t.get(s, []))}
only_t = {s: t[s] for s in t if t[s] and set(t[s]) - set(w.get(s, []))}
print('窗口臂多出来的（种子: 不变量）', only_w); print('今天臂多出来的', only_t)
