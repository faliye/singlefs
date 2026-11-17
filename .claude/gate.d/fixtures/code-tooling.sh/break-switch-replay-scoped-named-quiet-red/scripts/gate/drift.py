#!/usr/bin/env python3
# 样本脚本：自证分两格（alpha、beta）；环境变量 DRIFT_SELFTEST_CELLS=<格名,…> 只跑点名的格。
# 开关让 alpha 那一格判错，只在一句转述里带 ✗（不以 ✗ 起头），转述的格说的不是点名的那一句。
# 弄坏开关：
#   DRIFT_BREAK=relay  alpha 那一格判错，只在转述行的中间带 ✗
#       drift-relay（「✗ 自证：alpha：判前缀」）
import os
import sys

CELLS = {"alpha": "判前缀那一格", "beta": "判后缀那一格"}

if sys.argv[1:] == ["--selftest"]:
    selected = [name for name in os.environ.get("DRIFT_SELFTEST_CELLS", "").split(",") if name] or list(CELLS)
    if os.environ.get("DRIFT_BREAK") == "relay" and "alpha" in selected:
        print("  · 被测程序转述：✗ 自证：alpha：判后缀那一格：应当 1，实际 0")
        sys.exit(1)
    print(f"  ✓ 自证通过（查了 {len(selected)} 格）")
    sys.exit(0)
