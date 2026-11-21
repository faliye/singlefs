#!/usr/bin/env python3
# 样本脚本：divided.py 拆出的部分文件，读弄坏开关，开关表写在这里
# 弄坏开关：
#   DIVIDED_BREAK=drop  丢掉一格
#       divided-drop（「✗ 自检：丢了的那一格」）
import os


def selftest():
    if os.environ.get("DIVIDED_BREAK") == "drop":
        print("  ✗ 自检：丢了的那一格：应当 1，实际 0")
        return 1
    print("  ✓ 自检通过")
    return 0
