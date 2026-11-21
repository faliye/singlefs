#!/usr/bin/env python3
# 样本脚本：divided.py 拆出的部分文件，读两个弄坏开关；入口的文件头开关表里只有 DIVIDED_BREAK，ORPHAN_BREAK 没有
import os


def run():
    if os.environ.get("DIVIDED_BREAK") == "drop":
        print("  ✗ 自检：丢了的那一格")
    if os.environ.get("ORPHAN_BREAK") == "skip":
        print("  ✗ 自检：没人点名的那一格")
