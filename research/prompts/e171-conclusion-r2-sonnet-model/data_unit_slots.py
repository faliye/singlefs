#!/usr/bin/env python3
"""只读：列流二（固定脚本）盘 0 上落在单元区开头（槽 50176–50215）的写，槽号 = 偏移 ÷ 16384，只取到第 540 条 op。
用法：data_unit_slots.py <ops.tsv>   （列：序号 盘 种类 偏移 长度 载荷偏移 摘要）"""
import sys
for line in open(sys.argv[1], encoding="utf-8"):
    r = line.rstrip("\n").split("\t")
    if r[2] not in ("write", "write_fua") or r[1] != "0" or int(r[0]) > 540:
        continue
    slot = int(r[3]) // 16384
    if 50176 <= slot <= 50215:
        print(f"op={r[0]} slot={slot} bytes={r[4]}")
