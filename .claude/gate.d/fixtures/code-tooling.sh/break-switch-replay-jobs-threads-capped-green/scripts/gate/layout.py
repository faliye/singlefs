#!/usr/bin/env python3
# 样本脚本：并行度样本用的自证，不占内存、没有选格手段，两条开关各弄坏一格
# 弄坏开关：
#   LAYOUT_BREAK=prefix  判前缀那一处不判
#       layout-prefix（「✗ 自检：判前缀那一格」）
#   LAYOUT_BREAK=suffix  判后缀那一处不判
#       layout-suffix（「✗ 自检：判后缀那一格」）
import os
import sys

CELLS = {"prefix": "判前缀那一格", "suffix": "判后缀那一格"}

if sys.argv[1:] == ["--selftest"]:
    broken = os.environ.get("LAYOUT_BREAK", "")
    if broken in CELLS:
        print(f"  ✗ 自检：{CELLS[broken]}：应当 1，实际 0")
        sys.exit(1)
    print("  ✓ 自检通过（查了 2 格）")
    sys.exit(0)
