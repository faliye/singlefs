#!/usr/bin/env python3
# 样本脚本：名字与 lonely.py 起头相同的独立脚本，读 LONELY_BREAK；lonely.py 的代码里没有提它
import os

if os.environ.get("LONELY_BREAK") == "drop":
    print("  ✗ 自检：丢了的那一格")
