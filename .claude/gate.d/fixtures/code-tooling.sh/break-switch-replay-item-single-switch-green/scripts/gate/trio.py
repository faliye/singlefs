#!/usr/bin/env python3
# 样本脚本：不设开关时什么失败行都不打；三条开关各红一格。gamma 那一条点名的项写的是另一格，起了就判不符：只点 alpha 时它不该起
# 弄坏开关：
#   TRIO_BREAK=alpha  判甲那一处不判
#       trio-alpha（「✗ 自检：判甲那一格」）
#   TRIO_BREAK=beta  判乙那一处不判
#       trio-beta（「✗ 自检：判乙那一格」）
#   TRIO_BREAK=gamma  判丙那一处不判
#       trio-gamma（「✗ 自检：判丁那一格」）
import os
import sys

CELLS = {"alpha": "判甲那一格", "beta": "判乙那一格", "gamma": "判丙那一格"}

if sys.argv[1:] == ["--selftest"]:
    broken = os.environ.get("TRIO_BREAK")
    if broken in CELLS:
        print(f"  ✗ 自检：{CELLS[broken]}：应当 1，实际 0")
        sys.exit(1)
    print("  ✓ 自检通过（查了 3 格）")
    sys.exit(0)
