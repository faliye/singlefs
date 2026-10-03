#!/usr/bin/env python3
"""只读：把某一行某条流的 ops.tsv 与基线逐条比；报第一处不同的 op、不同条数、第一处的逐字节差（偏移、基线值、变异值）。
用法：diff_ops.py <基线流目录> <行流目录> [最多列几处字节差]"""
import sys

def load(d):
    ops = []
    for line in open(d + "/ops.tsv", encoding="utf-8"):
        p = line.rstrip("\n").split("\t")
        ops.append(p)
    payload = open(d + "/ops-payload.bin", "rb").read()
    return ops, payload

def main():
    base_dir, row_dir = sys.argv[1], sys.argv[2]
    limit = int(sys.argv[3]) if len(sys.argv) > 3 else 6
    bo, bp = load(base_dir)
    ro, rp = load(row_dir)
    print(f"ops_base={len(bo)} ops_row={len(ro)}")
    differing = [i for i in range(min(len(bo), len(ro))) if bo[i] != ro[i]]
    print(f"ops_differing_by_index={len(differing)} first={differing[0] if differing else None}")
    shown = 0
    for i in differing:
        b, r = bo[i], ro[i]
        if b[2] != "write" or r[2] != "write" or b[1:5] != r[1:5]:
            print(f"op {i}: base={b[:5]} row={r[:5]}  (kind/device/offset/length differ)")
            shown += 1
        else:
            length = int(b[4])
            bs = bp[int(b[5]):int(b[5]) + length]
            rs = rp[int(r[5]):int(r[5]) + length]
            diffs = [(k, bs[k], rs[k]) for k in range(length) if bs[k] != rs[k]]
            print(f"op {i}: write dev={b[1]} off={b[3]} len={length} bytes_differing={len(diffs)} " +
                  " ".join(f"@{k}:{x:#04x}->{y:#04x}" for k, x, y in diffs[:limit]))
            shown += 1
        if shown >= 4:
            break

main()
