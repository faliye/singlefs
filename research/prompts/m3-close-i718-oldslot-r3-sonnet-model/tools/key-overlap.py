#!/usr/bin/env python3
"""核「合法写坏镜像与坏写者在飞镜像在判器看得到的字段上逐字相同」：取 (N, t, F, p, 下一槽读不出, 记录点名 N+1) 当键，
比 B 格落洞的合法写坏镜像、F 格 p 写 255 的在飞 / 写坏镜像、A 格与 G 格的合法写坏镜像。用法：key-overlap.py <该 S 的合并文件>"""
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import tally  # noqa: E402

rows, _ = tally.load(sys.argv[1])
rows = [r for r in rows if "N" in r and r["img"] in ("zero", "torn_half")]


def key(r):
    return (r["N"], r["t"], r["F"], r["p"], r["next_unreadable"], r["rec_any"])


groups = {
    "B 落洞（合法）": {key(r) for r in rows if r["cell"] == "B" and r["today"] == "R"},
    "F p 写 255 在飞（坏写者）": {key(r) for r in rows if r["cell"] == "F" and r["hist"].endswith("/bad_p_255")},
    "A 写坏（合法）": {key(r) for r in rows if r["cell"] == "A" and r["today"] == "R"},
    "G 写行写坏（合法）": {key(r) for r in rows if r["cell"] == "G" and r["today"] == "R"},
}
for name, keys in groups.items():
    print(f"{name}: {len(keys)} 个键")
names = list(groups)
for i, a in enumerate(names):
    for b in names[i + 1:]:
        print(f"  {a} ∩ {b}: {len(groups[a] & groups[b])}")
