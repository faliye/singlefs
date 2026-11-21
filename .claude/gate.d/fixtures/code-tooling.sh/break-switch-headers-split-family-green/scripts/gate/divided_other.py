#!/usr/bin/env python3
# 样本脚本：只是名字与 divided.py 起头相同，divided.py 的代码里没有提它，是独立的脚本，自己写全开关表
# 弄坏开关：
#   OTHER_BREAK=skip  跳过一格
#       other-skip（「✗ 自检：跳过的那一格」）
import os

if os.environ.get("OTHER_BREAK") == "skip":
    print("  ✗ 自检：跳过的那一格")
