"""e171-conclusion-r3 攻方：E171 第 15 行与基线在新池新建文件那次发布（两条流的第 22–56 条 op）里每一次写的落点（16 KiB 槽号）逐条比。
用法：python3 row15_offsets.py <e171-dumps 目录>
"""
import os
import sys

dumps = sys.argv[1]
for stream in ("new_pool_file_creation", "fixed_script_through_e_then_unmount"):
    def rows(path):
        return [line.rstrip("\n").split("\t") for line in open(path, encoding="utf-8")]
    base = rows(os.path.join(dumps, "baseline-run1", stream, "ops.tsv"))
    mutant = rows(os.path.join(dumps, "rows-s4", "row-15", "dump", stream, "ops.tsv"))
    moved = []
    for b, m in zip(base[22:57], mutant[22:57]):
        if b[2] != "barrier" and b[1] == "0" and b[3] != m[3]:
            moved.append(f"op{b[0]}:{int(b[3]) // 16384}->{int(m[3]) // 16384}({b[4]})")
    print(f"name=row15_first_publish stream={stream} device0_writes_moved={len(moved)} " + " ".join(moved))
