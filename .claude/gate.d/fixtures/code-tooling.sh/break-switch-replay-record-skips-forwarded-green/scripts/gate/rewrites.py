#!/usr/bin/env python3
# 样本脚本：自证读同目录的 rewritten.tsv（进键），不设开关的那一趟读完往它末尾加一行：收尾记库时这一份的两条判定都「跑的过程中键变了」
# 弄坏开关：
#   REWRITES_BREAK=drop  丢掉一格
#       rewrites-drop（「✗ 自检：丢了的那一格」）
import os
import sys

TABLE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "rewritten.tsv")

if sys.argv[1:] == ["--selftest"]:
    with open(TABLE, encoding="utf-8") as handle:
        rows = handle.read().splitlines()
    if os.environ.get("REWRITES_BREAK") == "drop":
        print("  ✗ 自检：丢了的那一格：应当 1，实际 0")
        sys.exit(1)
    with open(TABLE, "a", encoding="utf-8") as handle:
        handle.write("appended\n")
    print(f"  ✓ 自检通过（表里原有 {len(rows)} 行）")
    sys.exit(0)
