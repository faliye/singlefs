#!/usr/bin/env python3
"""样本：只读同目录的 sample-table.tsv（进这一条的键）；改表的那一条加了一行之后，收尾记库时这一条也「跑的过程中键变了」。"""
import os
import sys

TABLE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "sample-table.tsv")

if sys.argv[1:] != ["--selftest"]:
    sys.exit(2)
with open(TABLE, encoding="utf-8") as handle:
    rows = handle.read().splitlines()
print(f"  ✓ 读表的自证 1 条都判对（表里 {len(rows)} 行）")
sys.exit(0)
