#!/usr/bin/env python3
"""样本脚本：开关那一条点名的项里「少一个收尾的括号，复跑读不到它点名的是哪一格
  UNCLOSED_BREAK=skip  判法整段跳过
    「✗ 自检：判法那一格
"""
import os
import sys

if sys.argv[1:] == ["--selftest"]:
    if os.environ.get("UNCLOSED_BREAK") == "skip":
        print("  ✗ 自检：判法那一格：应当 1，实际 0")
        sys.exit(1)
    print("  ✓ 自检通过")
    sys.exit(0)
