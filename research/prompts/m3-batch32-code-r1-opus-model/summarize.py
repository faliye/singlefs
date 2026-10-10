#!/usr/bin/env python3
"""从探针日志汇总：每条臂上 O3 族 loc0 与 loc1 红的集合不同的几形、I-4.9 / I-3.5 判定；分叉形各臂的 I-4.9 / I-3.5。
用法：summarize.py <日志目录>；输出排好序、不带耗时。"""
import re, sys, glob, os
lines = []
for path in sorted(glob.glob(os.path.join(sys.argv[1], "*.log"))):
    for line in open(path, encoding="utf-8", errors="replace"):
        i = line.find("PROBE ")
        if i >= 0:
            lines.append(line[i:].rstrip("\n"))
def field(line, name):
    m = re.search(r"\b" + name + r"=(\S+)", line)
    return m.group(1) if m else None
def verdict(line, name):
    m = re.search(r"\b" + name + r"=(H|NA|PJ|-|V<)", line)
    return {"V<": "V"}.get(m.group(1), m.group(1)) if m else None
out = []
o3 = {}
for line in lines:
    if line.startswith("PROBE p1o3 "):
        key = (field(line, "arm"), field(line, "base"), field(line, "walked"), field(line, "variant"))
        o3.setdefault(key, {})[field(line, "collide")] = (field(line, "red"), verdict(line, "i4_9"), verdict(line, "i3_5"))
for key in sorted(o3):
    arm, base, walked, variant = key
    l0 = o3[key].get("loc0"); l1 = o3[key].get("loc1")
    same = "SAME" if l0 == l1 else "DIFF"
    out.append(f"O3 arm={arm} base={base} walked={walked} variant={variant} loc0_vs_loc1={same} loc0_red={l0[0]} loc1_red={l1[0]} loc0_i4_9={l0[1]} loc0_i3_5={l0[2]}")
for line in lines:
    if line.startswith("PROBE p1split "):
        out.append("SPLIT arm={} direct={} raise_birth={} corrupt_loc0={} i4_9={} i3_5={} red={}".format(
            field(line, "arm"), field(line, "direct"), field(line, "raise_birth"), field(line, "corrupt_loc0"),
            verdict(line, "i4_9"), verdict(line, "i3_5"), field(line, "red")))
    elif line.startswith("PROBE p4 ") or line.startswith("PROBE p4const"):
        out.append(re.sub(r"i2_1=V<.*>", "i2_1=V", line.replace("PROBE ", "P4 ", 1)))
    elif line.startswith("PROBE p2 "):
        out.append(re.sub(r"i9_23=V<.*>", "i9_23=V", line.replace("PROBE ", "P2 ", 1)))
for line in sorted(out):
    print(line)
