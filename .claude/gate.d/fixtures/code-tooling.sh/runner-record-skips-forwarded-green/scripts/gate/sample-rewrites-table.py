#!/usr/bin/env python3
"""样本：读同目录的 sample-table.tsv（进这一条的键），读完往它末尾加一行：收尾记库时读这份表的每一条都「跑的过程中键变了」。"""
import os
import sys

TABLE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "sample-table.tsv")

if sys.argv[1:] != ["--selftest"]:
    sys.exit(2)
with open(TABLE, encoding="utf-8") as handle:
    rows = handle.read().splitlines()
with open(TABLE, "a", encoding="utf-8") as handle:
    handle.write("appended\n")
print(f"  ✓ 改表的自证 1 条都判对（表里原有 {len(rows)} 行）")
sys.exit(0)
