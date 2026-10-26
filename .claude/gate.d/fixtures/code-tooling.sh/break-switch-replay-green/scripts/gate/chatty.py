#!/usr/bin/env python3
# 样本脚本：只有一条开关；不设开关时自证也打一行带 ✗ 的拒绝行（被测的那一段本来就该打），开关下照样打，复跑要减掉它
# 弄坏开关：
#   CHATTY_BREAK=drop  丢掉一格
#       chatty-drop（「✗ 自检：丢了的那一格」）
import os
import sys

if sys.argv[1:] == ["--selftest"]:
    print("  ✗ 样本：被测的那一段本来就该打的拒绝行（自证照样过）")
    if os.environ.get("CHATTY_BREAK") == "drop":
        print("  ✗ 自检：丢了的那一格：应当 1，实际 0")
        sys.exit(1)
    print("  ✓ 自检通过")
    sys.exit(0)
