#!/usr/bin/env python3
# 样本脚本：divided.py 拆出的部分文件，读弄坏开关；开关表在入口 divided.py 的文件头
import os


def run():
    if os.environ.get("DIVIDED_BREAK") == "drop":
        print("  ✗ 自检：丢了的那一格")
