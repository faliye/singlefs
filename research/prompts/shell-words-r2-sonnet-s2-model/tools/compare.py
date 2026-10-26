#!/usr/bin/env python3
"""compare.py <新结果目录> <登记结果目录>：逐条比 id 与两份钩子的退出码、应当值；有一条对不上就退 1。"""
import sys; sys.dont_write_bytecode = True
import json, os, sys
new, old = sys.argv[1], sys.argv[2]
def load(d):
    out = {}
    for f in sorted(os.listdir(d)):
        if f.endswith('.jsonl'):
            for l in open(os.path.join(d, f)):
                r = json.loads(l); out[r['id']] = (r['rc_h'], r['rc_d'], r['exp_h'], r['exp_d'])
    return out
a, b = load(new), load(old)
diff = [(k, a.get(k), b.get(k)) for k in sorted(set(a) | set(b)) if a.get(k) != b.get(k)]
print(f"name=compare cases_new={len(a)} cases_registered={len(b)} differing={len(diff)}")
for k, x, y in diff[:30]:
    print('  ✗ 对不上', k, '新', x, '登记', y)
sys.exit(1 if diff else 0)
