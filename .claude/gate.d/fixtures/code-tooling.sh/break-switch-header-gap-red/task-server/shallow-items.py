#!/usr/bin/env python3
"""样本脚本：开关那一条的点名行缩进不够，读法认不出点名的项
  SHALLOW_BREAK=skip  判法整段跳过
  「✗ 自检：判法那一格」
"""
import os
import sys

if sys.argv[1:] == ["--selftest"]:
    if os.environ.get("SHALLOW_BREAK") == "skip":
        print("  ✗ 自检：判法那一格：应当 1，实际 0")
        sys.exit(1)
    print("  ✓ 自检通过")
    sys.exit(0)
