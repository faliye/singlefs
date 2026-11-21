#!/usr/bin/env python3
# 样本脚本：shared.py 拆出的部分文件，读弄坏开关，开关表写在这里
# 弄坏开关：
#   SHARED_BREAK=drop  丢掉一格
#       shared-drop（「✗ 自检：丢了的那一格」）
import os


def run():
    if os.environ.get("SHARED_BREAK") == "drop":
        print("  ✗ 自检：丢了的那一格")
