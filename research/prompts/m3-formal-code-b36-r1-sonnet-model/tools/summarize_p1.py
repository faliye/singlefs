#!/usr/bin/env python3
"""汇总 P1 子集矩阵（t-p1b.log）：每个改法下，checker 认下一次挂载先施加的那一版并红 I-5.4（detail 以「下一次挂载先施加的那一版」开头）
与 core 只读恢复施加 B（rec= 里 prefix=3）的交叉计数；再列不一致的行。用法 summarize_p1.py <log>"""
import collections, re, sys
rows = []
for l in open(sys.argv[1], encoding="utf-8"):
    m = re.match(r'PROBE-P1 mask=(\d+) variant=(\S+) red=(\[.*?\]) i54=(\w) i310=(\w) core=(.*?) rec=(.*?) d54=(.*)$', l.strip())
    if m: rows.append(m.groups())
print("行数", len(rows))
for variant in dict.fromkeys(r[1] for r in rows):
    c = collections.Counter(); odd = []
    for mask, v, red, i54, i310, core, rec, d54 in rows:
        if v != variant: continue
        checker = "checker 红 I-5.4（点名下一次挂载那一版）" if (i54 == "V" and d54.startswith("下一次挂载先施加的那一版")) else ("checker I-5.4 红在别处" if i54 == "V" else "checker 不红")
        core_applies = "prefix=3" in rec
        key = (checker, "core 施加 B" if core_applies else "core 不施加 B")
        c[key] += 1
        if (checker.startswith("checker 红 I-5.4（点名")) != core_applies and v.startswith("overlap") and v != "overlap+stale_last": odd.append((mask, key))  # stale_last：末条带的是 A 的根段，core 施加 B 而 B 自己的树不在被读的新根段里，按设计不红
    print(f"  {variant:28s}", dict(sorted(c.items())))
    if odd: print("    不一致:", odd[:8], "共", len(odd))

# I-3.10 与 I-5.4 是否同一个认法：overlap+gen 里两条逐掩码比（红的掩码集是否相同）
r54 = {m for m, v, red, i54, i310, core, rec, d54 in rows if v == "overlap+gen" and i54 == "V" and d54.startswith("下一次挂载先施加的那一版")}
r310 = {m for m, v, red, i54, i310, core, rec, d54 in rows if v == "overlap+gen" and i310 == "V"}
if any(v == "overlap+gen" for _, v, *_ in rows):
    allrows = [m for m, v, *_ in rows if v == "overlap+gen"]
    print("overlap+gen：掩码数", len(allrows), "I-5.4 点名下一次挂载那一版而红的掩码", len(r54), "I-3.10 红的掩码", len(r310), "两集合相等", r54 == r310)
    ov = {m for m, v, red, i54, i310, core, rec, d54 in rows if v == "overlap" and i54 == "V" and d54.startswith("下一次挂载先施加的那一版")}
    print("overlap 里 I-5.4 红的掩码集与 overlap+gen 里 I-3.10 红的掩码集相等", ov == r310)
