#!/usr/bin/env python3
# 样本脚本：一条开关红一格；复用三问里这一份一个字都不动
# 弄坏开关：
#   BETA_BREAK=skip  跳过一格
#       beta-skip（「✗ 自检：跳过的那一格」）
import os
import sys

if sys.argv[1:] == ["--selftest"]:
    if os.environ.get("BETA_BREAK") == "skip":
        print("  ✗ 自检：跳过的那一格：应当判过，实际没判")
        sys.exit(1)
    print("  ✓ 自检通过")
    sys.exit(0)
