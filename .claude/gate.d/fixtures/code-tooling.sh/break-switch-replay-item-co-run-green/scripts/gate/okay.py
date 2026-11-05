#!/usr/bin/env python3
# 样本脚本：弄坏开关红在点名的失败行上（这一份样本里没被点名，不该起）
# 弄坏开关：
#   OKAY_BREAK=drop  丢掉一格
#       okay-drop（「✗ 自检：丢了的那一格」）
import os
import sys

if sys.argv[1:] == ["--selftest"]:
    if os.environ.get("OKAY_BREAK") == "drop":
        print("  ✗ 自检：丢了的那一格：应当 1，实际 0")
        sys.exit(1)
    print("  ✓ 自检通过")
    sys.exit(0)
