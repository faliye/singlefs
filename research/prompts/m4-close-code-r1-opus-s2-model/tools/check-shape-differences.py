"""攻方腿 s2：核容器树形状那一段改前改后的不同，是不是全是「某一层孩子数之和越过 u64」那一种（改前 panic 或收下、改后交 None）。
形状那一段改前改后收下的集合不同，之后取样的输入跟着岔开，所以按输入配对，只比两边都取到的输入。参数：草稿目录。"""
import json
import re
import sys

work = sys.argv[1]


def shapes(side):
    found = {}
    for line in open(f"{work}/out-tree-{side}.txt", encoding="utf-8"):
        parts = line.split(" ", 2)
        if parts[1] == "shape":
            matched = re.match(r"\((\[.*?\]\]|\[\]), (.*)\)\n", parts[2])
            found.setdefault(matched.group(1), set()).add(matched.group(2))
    return found


old = shapes("old")
new = shapes("new")
both = [key for key in old if key in new]
differing = [key for key in both if old[key] != new[key]]
print("shape inputs old", len(old), "new", len(new), "in both", len(both), "differing", len(differing))
kinds = {}
unexplained = 0
for key in differing:
    levels = json.loads(key)
    overflowing = any(sum(level) >= 2**64 for level in levels)
    old_result = sorted(old[key])[0][:12]
    if overflowing and new[key] == {"Ok(None)"}:
        kinds[old_result] = kinds.get(old_result, 0) + 1
    else:
        unexplained += 1
        if unexplained < 5:
            print("UNEXPLAINED", key, old[key], new[key])
print("explained by an overflowing level sum (old result -> count):", kinds)
print("unexplained", unexplained)
