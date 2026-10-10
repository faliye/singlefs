#!/usr/bin/env python3
"""K1 用户动作放开扫：固定前缀（含故障点），后缀在字母表上全枚举，切成 <件数> 份 tsv。
用法：gen_sweep.py <输出目录> <件数>"""
import itertools, os, sys

out, parts = sys.argv[1], int(sys.argv[2])
plans = [
    # (前缀名, 前缀串, 字母表, 后缀长)
    ("p233", "U C*233", ["C", "R1", "R2", "M", "X", "K", "F1"], 3),
    ("p234", "U C*234", ["C", "R1", "R2", "M", "X", "K", "F1"], 3),
    ("p233", "U C*233", ["C", "R1", "M", "X"], 4),
    ("p234", "U C*234", ["C", "R1", "M", "X"], 4),
    ("p466", "U C*466", ["C", "R1", "R2", "M", "X", "K", "F1"], 2),
    ("p467", "U C*467", ["C", "R1", "R2", "M", "X", "K", "F1"], 2),
]
rows, seen = [], set()
for name, prefix, alphabet, length in plans:
    for suffix in itertools.product(alphabet, repeat=length):
        ops = prefix + " " + " ".join(suffix)
        if ops in seen:
            continue
        seen.add(ops)
        rows.append(f"{name}-{'-'.join(suffix)}\t{ops}")
os.makedirs(out, exist_ok=True)
for i in range(parts):
    with open(os.path.join(out, f"part{i}.tsv"), "x") as f:
        f.write("\n".join(rows[i::parts]) + "\n")
print(f"histories={len(rows)} parts={parts}")
