#!/usr/bin/env python3
# 样本脚本：文件头写全、登记成 light，却还没接进复跑登记表
# 弄坏开关：
#   OTHER_BREAK=skip  跳过一格
#       other-skip（「✗ 自检：跳过的那一格」）
import os
import sys

if sys.argv[1:] == ["--selftest"]:
    if os.environ.get("OTHER_BREAK") == "skip":
        print("  ✗ 自检：跳过的那一格")
        sys.exit(1)
    print("  ✓ 自检通过")
    sys.exit(0)
