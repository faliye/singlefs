#!/usr/bin/env python3
"""从 summarize.py 的输出出一张逐 (宽, k, 臂) 的表：第 0 步被写掉的落点、卸载、再挂、探针；之后哪一步起卸载 / 探针变了；覆盖写第几次被拒。"""
import re, sys
cur = None
out = []
for line in open(sys.argv[1]):
    if line.startswith("== "):
        cur = {"key": line[3:].strip(), "steps": []}
        out.append(cur)
    else:
        cur["steps"].append(line.strip())
for row in out:
    first = row["steps"][0]
    fails = [s for s in row["steps"] if "OVERWRITE_FAILED" in s]
    probes = sorted(set(re.findall(r"probe=(\S+)", " ".join(row["steps"]))))
    ums = sorted(set(re.findall(r" um=(\S+)", " ".join(row["steps"]))))
    print(f"{row['key']} | step0 {first.split(':',1)[1].strip()} | 各步卸载={ums} 探针={probes} | {fails[0] if fails else '覆盖写 0..14 都做成'}")
