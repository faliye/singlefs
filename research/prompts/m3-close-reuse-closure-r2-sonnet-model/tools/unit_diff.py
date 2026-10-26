#!/usr/bin/env python3
"""两份 closure --explain 输出的单元差：键取（种类，限定名，文件，指纹），行号不比。用法：unit_diff.py <前> <后>"""
import collections, sys
def units(path):
    s = {}
    for l in open(path, encoding="utf-8"):
        p = l.rstrip("\n").split("\t")
        if p[0] in ("via", "key", "refused") or len(p) < 4:
            continue
        s[(p[0], p[1], p[2].rsplit(":", 1)[0], p[3])] = p[2]
    return s
a, b = units(sys.argv[1]), units(sys.argv[2])
lost = sorted(set(a) - set(b)); gained = sorted(set(b) - set(a))
print(f"前 {len(a)} 后 {len(b)} 少 {len(lost)} 多 {len(gained)}")
for f, n in collections.Counter(x[2] for x in lost).most_common():
    print(f"  少 {n:4d} {f}")
for f, n in collections.Counter(x[2] for x in gained).most_common():
    print(f"  多 {n:4d} {f}")
if len(sys.argv) > 3:
    for x in lost:
        print("LOST", x[2], x[0], x[1])
