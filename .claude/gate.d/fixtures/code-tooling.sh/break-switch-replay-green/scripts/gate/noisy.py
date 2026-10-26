#!/usr/bin/env python3
# 样本脚本：不设开关时自证也打一行带 ✗ 的拒绝行（被测的那一段本来就该打），三条开关各红一格、另带一句汇总行
# 弄坏开关：
#   NOISY_BREAK=prefix  判前缀那一处不判
#       noisy-prefix（「✗ 自检：判前缀那一格」）
#   NOISY_BREAK=suffix  判后缀那一处不判
#       noisy-suffix（「✗ 自检：判后缀那一格」）
#   NOISY_BREAK=length  判长度那一处不判
#       noisy-length（「✗ 自检：判长度那一格」）
import os
import sys

CELLS = {"prefix": "判前缀那一格", "suffix": "判后缀那一格", "length": "判长度那一格"}

if sys.argv[1:] == ["--selftest"]:
    print("  ✗ 样本：被测的那一段本来就该打的拒绝行（自证照样过）")
    broken = os.environ.get("NOISY_BREAK")
    if broken in CELLS:
        print(f"  ✗ 自检：{CELLS[broken]}：应当 1，实际 0")
        print("  ✗ 自检：1 格不对（查了 3 格）")
        sys.exit(1)
    print("  ✓ 自检通过（查了 3 格）")
    sys.exit(0)
