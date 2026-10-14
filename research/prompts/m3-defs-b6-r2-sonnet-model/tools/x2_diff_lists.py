#!/usr/bin/env python3
"""X2：三份判法（after / before / pre0）在同一棵树（after 的拷贝，排除表清空）上的被判行集合，两两求差。用法：x2_diff_lists.py <WORK>（要有 x2-empty-<版本>.txt）"""
import re, sys
work = sys.argv[1]
def load(version):
    found = {}
    for line in open(f"{work}/x2-empty-{version}.txt", encoding="utf-8"):
        match = re.match(r"     (\S+?):(\d+)  (.*?)  「(.*)」$", line.rstrip("\n"))
        if match:
            found[(match.group(1), match.group(2), match.group(4)[:80])] = match.group(3)
    return found
data = {v: load(v) for v in ("after", "before", "pre0")}
print("被判行数（路径、行号、行文前 80 字去重）：", {v: len(d) for v, d in data.items()})
for left, right in (("after", "before"), ("before", "after"), ("after", "pre0"), ("pre0", "after")):
    extra = sorted(set(data[left]) - set(data[right]))
    print(f"== {left} 有而 {right} 没有 {len(extra)} 行")
    for path, number, text in extra:
        print(f"   {path}:{number}  {data[left][(path, number, text)][:70]}  | {text[:90]}")
