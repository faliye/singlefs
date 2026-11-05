#!/usr/bin/env python3
# 样本脚本：开关设上时红在点名的那一格；当前目录（被判的样本根）有 .red-switch 时改红在没点名的另一格
# 弄坏开关：
#   FLAKY_BREAK=swap  两格换了位置
#       flaky-swap（「✗ 自检：点名的那一格」）
import os
import sys

if sys.argv[1:] == ["--selftest"]:
    if os.environ.get("FLAKY_BREAK") == "swap":
        if os.path.exists(".red-switch"):
            print("  ✗ 自检：另一格：应当 1，实际 0")
        else:
            print("  ✗ 自检：点名的那一格：应当 1，实际 0")
        sys.exit(1)
    print("  ✓ 自检通过")
    sys.exit(0)
