#!/usr/bin/env python3
# 样本脚本：开关打开后自证红了两格，文件头只点名一格（多红）
# 弄坏开关：
#   PARTIAL_BREAK=two  两格一起弄坏
#       partial-two（「✗ 自检：点名的那一格」）
import os
import sys

if sys.argv[1:] == ["--selftest"]:
    if os.environ.get("PARTIAL_BREAK") == "two":
        print("  ✗ 自检：点名的那一格：应当 1，实际 0")
        print("  ✗ 自检：没点名的那一格：应当 1，实际 0")
        sys.exit(1)
    print("  ✓ 自检通过")
    sys.exit(0)
