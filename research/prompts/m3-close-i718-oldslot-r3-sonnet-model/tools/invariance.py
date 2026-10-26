#!/usr/bin/env python3
"""各 S 的分类键集合比对。键 = (格, 历史标签, 镜像种类, 下一槽读不出, 记录点名 N+1, N+1>=3S, 七个候选的判定向量)。
用法：invariance.py <all-S4.txt> <all-S5.txt> …（一个 S 一份）。只在有行的 S 之间比。"""
import sys
from collections import defaultdict

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import tally  # noqa: E402


def keyset(path):
    rows, _ = tally.load(path)
    keys = set()
    s = None
    for r in rows:
        if "N" not in r:
            continue
        s = int(r["S"])
        label = r["hist"].split("/")[-1] if "/" in r["hist"] else "-"
        if r["cell"] in ("A", "BR", "Y", "G", "B") or r["cell"].startswith("Z2"):
            label = label if label in ("honest", "bad_t_as_floor", "bad_p_255") else "-"
        big = int(r["N"]) + 1 >= 3 * s
        img = "damaged" if r["img"] in ("zero", "torn_half", "torn_enum") else r["img"]
        keys.add((r["cell"], label, img, r["next_unreadable"], r["rec_any"], big, tally.vector(r)))
    return s, keys


def main():
    per_s = {}
    for path in sys.argv[1:]:
        s, keys = keyset(path)
        per_s[s] = keys
    all_keys = set().union(*per_s.values())
    print("S 取值：", sorted(per_s), "各 S 键数：", {s: len(k) for s, k in sorted(per_s.items())}, "并集键数：", len(all_keys))
    full = [s for s in per_s if s in (4, 5, 8)]
    common = set.intersection(*(per_s[s] for s in full))
    print(f"S={full} 三个都有的键 {len(common)} 个；并集 {len(set().union(*(per_s[s] for s in full)))} 个")
    only = defaultdict(list)
    for key in set().union(*(per_s[s] for s in full)):
        present = tuple(s for s in sorted(full) if key in per_s[s])
        if len(present) != len(full):
            only[present].append(key)
    for present, keys in sorted(only.items()):
        print("只在", present, "出现的键", len(keys))
        for key in sorted(keys)[:12]:
            print("   ", key)
    for s in sorted(per_s):
        if s not in full:
            missing = per_s[s] - set().union(*(per_s[t] for t in full))
            print(f"S={s} 的键里 S=4、5、8 都没有的：{len(missing)} 个", sorted(missing)[:6])


if __name__ == "__main__":
    main()
